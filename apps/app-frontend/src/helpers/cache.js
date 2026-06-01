import { invoke } from '@tauri-apps/api/core'
import { isCurseForgeId, isCurseForgeVersionId, getProject, getVersionsMany } from './curseforge.js'

export async function get_project(id, cacheBehaviour) {
  if (isCurseForgeId(id)) {
    return await getProject(id)
  }
  return await invoke('plugin:cache|get_project', { id, cacheBehaviour })
}

export async function get_project_many(ids, cacheBehaviour) {
  const cfIds = ids.filter(isCurseForgeId)
  const mrIds = ids.filter(id => !isCurseForgeId(id))
  
  let results = []
  if (cfIds.length > 0) {
    const cfProjects = await Promise.all(cfIds.map(getProject))
    results.push(...cfProjects)
  }
  if (mrIds.length > 0) {
    const mrProjects = await invoke('plugin:cache|get_project_many', { ids: mrIds, cacheBehaviour })
    results.push(...mrProjects)
  }
  return results
}

export async function get_version(id, cacheBehaviour) {
  if (isCurseForgeVersionId(id)) {
    const res = await getVersionsMany([id])
    return res[0]
  }
  return await invoke('plugin:cache|get_version', { id, cacheBehaviour })
}

export async function get_version_many(ids, cacheBehaviour) {
  const cfIds = ids.filter(isCurseForgeVersionId)
  const mrIds = ids.filter(id => !isCurseForgeVersionId(id))
  
  let results = []
  if (cfIds.length > 0) {
    const cfVersions = await getVersionsMany(cfIds)
    results.push(...cfVersions)
  }
  if (mrIds.length > 0) {
    const mrVersions = await invoke('plugin:cache|get_version_many', { ids: mrIds, cacheBehaviour })
    results.push(...mrVersions)
  }
  return results
}

export async function get_user(id, cacheBehaviour) {
  return await invoke('plugin:cache|get_user', { id, cacheBehaviour })
}

export async function get_user_many(ids, cacheBehaviour) {
  return await invoke('plugin:cache|get_user_many', { ids, cacheBehaviour })
}

export async function get_team(id, cacheBehaviour) {
  return await invoke('plugin:cache|get_team', { id, cacheBehaviour })
}

export async function get_team_many(ids, cacheBehaviour) {
  return await invoke('plugin:cache|get_team_many', { ids, cacheBehaviour })
}

export async function get_organization(id, cacheBehaviour) {
  return await invoke('plugin:cache|get_organization', { id, cacheBehaviour })
}

export async function get_organization_many(ids, cacheBehaviour) {
  return await invoke('plugin:cache|get_organization_many', { ids, cacheBehaviour })
}

export async function get_search_results(id, cacheBehaviour) {
  return await invoke('plugin:cache|get_search_results', { id, cacheBehaviour })
}

export async function get_search_results_many(ids, cacheBehaviour) {
  return await invoke('plugin:cache|get_search_results_many', { ids, cacheBehaviour })
}

export async function purge_cache_types(cacheTypes) {
  return await invoke('plugin:cache|purge_cache_types', { cacheTypes })
}
