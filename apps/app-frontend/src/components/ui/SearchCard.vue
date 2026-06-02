<template>
  <div
    class="card-shadow p-4 bg-bg-raised rounded-xl flex gap-3 group cursor-pointer hover:brightness-90 transition-all"
    :class="{ 'opacity-60': props.instance && !isCompatible }"
    @click="
      () => {
        emit('open')
        $router.push({
          path: `/project/${project.project_id ?? project.id}`,
          query: { i: props.instance ? props.instance.path : undefined },
        })
      }
    "
  >
    <div class="icon w-[96px] h-[96px] relative">
      <Avatar :src="project.icon_url" size="96px" class="search-icon origin-top transition-all" />
    </div>
    <div class="flex flex-col gap-2 overflow-hidden w-full">
      <div class="gap-2 overflow-hidden no-wrap text-ellipsis flex items-baseline">
        <span class="text-lg font-extrabold text-contrast m-0 leading-none">
          {{ project.title }}
        </span>
        <span v-if="project.author" class="text-secondary"> by {{ project.author }}</span>
      </div>
      <div
        v-if="props.instance && !isCompatible"
        class="text-xs font-bold px-2 py-0.5 rounded w-fit uppercase tracking-wider"
        style="background-color: var(--color-red-bg); color: var(--color-red); border: 1px solid var(--color-red-highlight);"
      >
        Not available for {{ props.instance.game_version }} {{ props.project.project_type === 'mod' && ['fabric', 'forge', 'quilt', 'neoforge'].includes(props.instance.loader) ? props.instance.loader : '' }}
      </div>
      <div class="m-0 line-clamp-2">
        {{ project.description }}
      </div>
      <div v-if="categories.length > 0" class="mt-auto flex items-center gap-1 no-wrap">
        <TagsIcon class="h-4 w-4 shrink-0" />
        <div
          v-if="project.project_type === 'mod' || project.project_type === 'modpack'"
          class="text-sm font-semibold text-secondary flex gap-1 px-[0.375rem] py-0.5 bg-button-bg rounded-full"
        >
          <template v-if="project.client_side === 'optional' && project.server_side === 'optional'">
            Client or server
          </template>
          <template
            v-else-if="
              (project.client_side === 'optional' || project.client_side === 'required') &&
              (project.server_side === 'optional' || project.server_side === 'unsupported')
            "
          >
            Client
          </template>
          <template
            v-else-if="
              (project.server_side === 'optional' || project.server_side === 'required') &&
              (project.client_side === 'optional' || project.client_side === 'unsupported')
            "
          >
            Server
          </template>
          <template
            v-else-if="
              project.client_side === 'unsupported' && project.server_side === 'unsupported'
            "
          >
            Unsupported
          </template>
          <template
            v-else-if="project.client_side === 'required' && project.server_side === 'required'"
          >
            Client and server
          </template>
        </div>
        <div
          v-for="tag in categories"
          :key="tag"
          class="text-sm font-semibold text-secondary flex gap-1 px-[0.375rem] py-0.5 bg-button-bg rounded-full"
        >
          {{ formatCategory(tag.name) }}
        </div>
      </div>
    </div>
    <div class="flex flex-col gap-2 items-end shrink-0 ml-auto">
      <div class="flex items-center gap-2">
        <DownloadIcon class="shrink-0" />
        <span>
          {{ formatNumber(project.downloads) }}
          <span class="text-secondary">downloads</span>
        </span>
      </div>
      <div class="flex items-center gap-2">
        <HeartIcon class="shrink-0" />
        <span>
          {{ formatNumber(project.follows ?? project.followers) }}
          <span class="text-secondary">followers</span>
        </span>
      </div>
      <div class="mt-auto relative">
        <div class="absolute bottom-0 right-0 w-fit">
          <ButtonStyled :color="isCompatible ? 'brand' : 'raised'" type="outlined">
            <button
              :disabled="installed || installing || (props.instance && !isCompatible)"
              class="shrink-0 no-wrap"
              @click.stop="install()"
            >
              <template v-if="!installed">
                <DownloadIcon v-if="modpack || (instance && isCompatible)" />
                <PlusIcon v-else />
              </template>
              <CheckIcon v-else />
              {{
                installing
                  ? 'Installing'
                  : installed
                    ? 'Installed'
                    : props.instance && !isCompatible
                      ? 'Not Available'
                      : modpack || instance
                        ? 'Install'
                        : 'Add to an instance'
              }}
            </button>
          </ButtonStyled>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { TagsIcon, DownloadIcon, HeartIcon, PlusIcon, CheckIcon } from '@modrinth/assets'
import { ButtonStyled, Avatar } from '@modrinth/ui'
import { formatNumber, formatCategory } from '@modrinth/utils'
import dayjs from 'dayjs'
import relativeTime from 'dayjs/plugin/relativeTime'
import { ref, computed } from 'vue'
import { install as installVersion } from '@/store/install.js'
dayjs.extend(relativeTime)

const props = defineProps({
  backgroundImage: {
    type: String,
    default: null,
  },
  project: {
    type: Object,
    required: true,
  },
  categories: {
    type: Array,
    required: true,
  },
  instance: {
    type: Object,
    default: null,
  },
  featured: {
    type: Boolean,
    default: false,
  },
  installed: {
    type: Boolean,
    default: false,
  },
})

const emit = defineEmits(['open', 'install'])

const installing = ref(false)

async function install() {
  if (props.instance && !isCompatible.value) return
  installing.value = true
  await installVersion(
    props.project.project_id ?? props.project.id,
    null,
    props.instance ? props.instance.path : null,
    'SearchCard',
    () => {
      installing.value = false
      emit('install', props.project.project_id ?? props.project.id)
    },
  )
}

const modpack = computed(() => props.project.project_type === 'modpack')

const isCompatible = computed(() => {
  if (!props.instance) return true

  const gameVersion = props.instance.game_version
  const loader = props.instance.loader

  // Check game version compatibility
  if (props.project.project_type !== 'modpack') {
    if (props.project.versions && props.project.versions.length > 0) {
      const hasGameVersion = props.project.versions.includes(gameVersion)
      if (!hasGameVersion) return false
    }
  }

  // Check loader compatibility for mods
  if (props.project.project_type === 'mod') {
    if (loader === 'vanilla') return false
    const supportedModLoaders = ['fabric', 'forge', 'quilt', 'neoforge']
    if (loader && supportedModLoaders.includes(loader)) {
      const hasLoader = props.project.display_categories && props.project.display_categories.includes(loader)
      if (!hasLoader) return false
    }
  }

  return true
})
</script>
