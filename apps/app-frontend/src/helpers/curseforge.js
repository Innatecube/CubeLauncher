import { invoke } from '@tauri-apps/api/core'

const API_KEY = '$2a$10$GfFWszfepvMttJqMe8l0Z.LzpuKuhLNj7CZU6o0smcOUKYiNRf0ye'
const BASE_URL = 'https://api.curseforge.com/v1'

const HEADERS = {
  'Accept': 'application/json',
  'x-api-key': API_KEY,
}

// Local cache for CurseForge versions so we don't have to re-fetch them constantly
const versionsCache = new Map()

// Maps Modrinth project type names to CurseForge classIds
function getCFClassId(projectType) {
  switch (projectType) {
    case 'modpack':
      return 4471
    case 'mod':
      return 6
    case 'resourcepack':
      return 12
    case 'shader':
    case 'datapack':
      return 4546 // Customization
    default:
      return 6
  }
}

// Maps Modrinth loader name to CurseForge modLoaderType enum
function getCFModLoaderType(loader) {
  if (!loader) return undefined
  const l = loader.toLowerCase()
  if (l.includes('forge') && !l.includes('neo')) return 1
  if (l.includes('fabric')) return 4
  if (l.includes('quilt')) return 5
  if (l.includes('neoforge')) return 6
  return undefined
}

// Maps CurseForge classId to Modrinth project type name
function getModrinthProjectType(classId) {
  switch (classId) {
    case 4471:
      return 'modpack'
    case 6:
      return 'mod'
    case 12:
      return 'resourcepack'
    case 4546:
      return 'customization'
    default:
      return 'mod'
  }
}

export function isCurseForgeId(id) {
  return typeof id === 'string' && id.startsWith('cf-')
}

export function isCurseForgeVersionId(id) {
  return typeof id === 'string' && id.startsWith('cfv-')
}

export function parseCurseForgeId(id) {
  if (isCurseForgeId(id)) {
    return parseInt(id.substring(3), 10)
  }
  return id
}

export function parseCurseForgeVersionId(id) {
  if (isCurseForgeVersionId(id)) {
    return parseInt(id.substring(4), 10)
  }
  return id
}

// Map CurseForge mod result to Modrinth search hit schema
function mapCFModToSearchHit(mod) {
  const categories = mod.categories?.map(c => c.name.toLowerCase()) || []
  
  // Try to determine loaders
  const loaders = []
  if (mod.latestFilesIndexes) {
    for (const f of mod.latestFilesIndexes) {
      const lt = f.modLoader
      if (lt === 1) loaders.push('forge')
      if (lt === 4) loaders.push('fabric')
      if (lt === 5) loaders.push('quilt')
      if (lt === 6) loaders.push('neoforge')
    }
  }
  
  const display_categories = [...categories, ...loaders]

  return {
    project_id: `cf-${mod.id}`,
    id: `cf-${mod.id}`,
    slug: mod.slug,
    title: mod.name,
    description: mod.summary,
    icon_url: mod.logo?.url || mod.logo?.thumbnailUrl || '',
    downloads: mod.downloadCount || 0,
    follows: mod.thumbsUpCount || 0,
    followers: mod.thumbsUpCount || 0,
    author: mod.authors?.map(a => a.name).join(', ') || 'Unknown',
    project_type: getModrinthProjectType(mod.classId),
    display_categories: display_categories,
    client_side: 'optional',
    server_side: 'optional',
    versions: [], // filled if requested
    installed: false,
  }
}

// Map CurseForge file to Modrinth Version schema
function mapCFFileToVersion(file, projectId) {
  const sha1Obj = file.hashes?.find(h => h.algo === 1)
  const sha1 = sha1Obj ? sha1Obj.value : ''
  
  // Extract loaders
  const loaders = []
  if (file.gameVersions) {
    file.gameVersions.forEach(gv => {
      const lower = gv.toLowerCase()
      if (['forge', 'fabric', 'quilt', 'neoforge'].includes(lower)) {
        loaders.push(lower)
      }
    })
  }

  // Filter game versions (e.g. 1.20.1)
  const gameVersions = file.gameVersions?.filter(gv => {
    // Standard version numbers containing dots, e.g. 1.20.1
    return /^\d+\.\d+/.test(gv)
  }) || []

  return {
    id: `cfv-${file.id}`,
    project_id: projectId,
    version_number: file.displayName || file.fileName,
    name: file.displayName || file.fileName,
    date_published: file.fileDate,
    downloads: file.downloadCount || 0,
    game_versions: gameVersions,
    loaders: loaders.length > 0 ? loaders : ['minecraft'],
    files: [
      {
        id: `cff-${file.id}`,
        url: file.downloadUrl,
        filename: file.fileName,
        primary: true,
        size: file.fileLength,
        hashes: {
          sha1: sha1,
        }
      }
    ],
    dependencies: file.dependencies?.map(d => ({
      project_id: `cf-${d.modId}`,
      dependency_type: d.relationType === 3 ? 'required' : 'optional',
    })) || [],
  }
}


/**
 * Searches CurseForge mods.
 * requestParams is a URL query string like "?limit=20&index=relevance&query=...&facets=..."
 * as produced by the useSearch() composable in @modrinth/ui.
 */
export async function search(requestParamsStr) {
  try {
    // Parse the query string
    const qs = typeof requestParamsStr === 'string'
      ? new URLSearchParams(requestParamsStr.startsWith('?') ? requestParamsStr.slice(1) : requestParamsStr)
      : new URLSearchParams()

    const queryText = qs.get('query') ? decodeURIComponent(qs.get('query')) : ''
    const indexName = qs.get('index') || 'relevance'
    const limit = parseInt(qs.get('limit') || '20', 10)
    const offset = parseInt(qs.get('offset') || '0', 10)

    // Parse facets to extract project_type, loaders, game versions
    let projectType = 'mod'
    let modLoaderType = undefined
    let gameVersion = undefined

    try {
      const facetsRaw = qs.get('facets')
      if (facetsRaw) {
        const facets = JSON.parse(decodeURIComponent(facetsRaw))
        for (const group of facets) {
          for (const facet of group) {
            if (typeof facet === 'string') {
              if (facet.startsWith('project_type:')) {
                projectType = facet.replace('project_type:', '')
              } else if (facet.startsWith('categories:')) {
                const loader = facet.replace('categories:', '')
                const lt = getCFModLoaderType(loader)
                if (lt) modLoaderType = lt
              } else if (facet.startsWith('versions:')) {
                gameVersion = facet.replace('versions:', '')
              }
            }
          }
        }
      }
    } catch (e) {
      console.warn('[CF] Could not parse facets:', e)
    }

    const classId = getCFClassId(projectType)

    // Sort field mapping
    let sortField = 2 // Popularity
    if (indexName === 'relevance' || indexName === 'newest') sortField = 1
    else if (indexName === 'updated') sortField = 3
    else if (indexName === 'downloads') sortField = 6

    const params = new URLSearchParams({
      gameId: '432',
      classId: classId.toString(),
      sortField: sortField.toString(),
      sortOrder: 'desc',
      index: offset.toString(),
      pageSize: limit.toString(),
    })

    if (queryText) params.append('searchFilter', queryText)
    if (modLoaderType !== undefined) params.append('modLoaderType', modLoaderType.toString())
    if (gameVersion) params.append('gameVersion', gameVersion)

    const res = await fetch(`${BASE_URL}/mods/search?${params.toString()}`, {
      headers: HEADERS,
    })

    if (!res.ok) {
      throw new Error(`CurseForge Search failed: ${res.status} ${res.statusText}`)
    }

    const data = await res.json()
    const hits = (data.data || []).map(mapCFModToSearchHit)

    return {
      result: {
        hits,
        total_hits: data.pagination?.totalCount || hits.length,
        limit,
      },
    }
  } catch (error) {
    console.error('[CF] Search error:', error)
    return {
      result: { hits: [], total_hits: 0, limit: 20 },
    }
  }
}

/**
 * Gets a single CurseForge project details
 */
export async function getProject(id) {
  const cfId = parseCurseForgeId(id)
  const res = await fetch(`${BASE_URL}/mods/${cfId}`, {
    headers: HEADERS,
  })
  
  if (!res.ok) {
    throw new Error(`CurseForge getProject failed: ${res.statusText}`)
  }

  const data = await res.json()
  const mod = data.data
  const hit = mapCFModToSearchHit(mod)
  
  // Fetch files (versions)
  const filesRes = await fetch(`${BASE_URL}/mods/${cfId}/files`, {
    headers: HEADERS,
  })
  if (filesRes.ok) {
    const filesData = await filesRes.json()
    const mappedVersions = (filesData.data || []).map(file => mapCFFileToVersion(file, `cf-${cfId}`))
    mappedVersions.forEach(v => {
      versionsCache.set(v.id, v)
    })
    hit.versions = mappedVersions.map(v => v.id)
  }
  
  hit.body = mod.description || mod.summary
  hit.gallery = mod.screenshots?.map(s => ({
    url: s.url,
    title: s.title,
    description: s.description,
  })) || []
  
  hit.team = `cf-team-${mod.id}`
  hit.slug = mod.slug
  hit.client_side = 'optional'
  hit.server_side = 'optional'
  
  return hit
}

/**
 * Gets all files (versions) for a CurseForge project
 */
export async function getVersions(id) {
  const cfId = parseCurseForgeId(id)
  const res = await fetch(`${BASE_URL}/mods/${cfId}/files`, {
    headers: HEADERS,
  })

  if (!res.ok) {
    throw new Error(`CurseForge getVersions failed: ${res.statusText}`)
  }

  const data = await res.json()
  const mappedVersions = (data.data || []).map(file => mapCFFileToVersion(file, `cf-${cfId}`))
  mappedVersions.forEach(v => {
    versionsCache.set(v.id, v)
  })
  return mappedVersions
}

/**
 * Gets a list of specific versions by their ids
 */
export async function getVersionsMany(versionIds) {
  const missingIds = []
  const results = []
  
  versionIds.forEach(id => {
    if (versionsCache.has(id)) {
      results.push(versionsCache.get(id))
    } else {
      missingIds.push(id)
    }
  })
  
  if (missingIds.length > 0) {
    const cleanIds = missingIds.map(parseCurseForgeVersionId)
    const res = await fetch(`${BASE_URL}/mods/files`, {
      method: 'POST',
      headers: {
        ...HEADERS,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ fileIds: cleanIds }),
    })

    if (res.ok) {
      const data = await res.json()
      const fetched = (data.data || []).map(file => mapCFFileToVersion(file, `cf-${file.modId}`))
      fetched.forEach(v => {
        versionsCache.set(v.id, v)
        results.push(v)
      })
    }
  }
  
  return results
}
