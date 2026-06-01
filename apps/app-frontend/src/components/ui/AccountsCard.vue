<template>
  <div
    v-if="mode !== 'isolated'"
    ref="button"
    class="button-base mt-2 px-3 py-2 bg-button-bg rounded-xl flex items-center gap-2"
    :class="{ expanded: mode === 'expanded' }"
    @click="toggleMenu"
  >
    <Avatar v-if="!selectedAccount || !isElyBy(selectedAccount)"
      size="36px"
      :src="selectedAccount ? getAvatarUrl(selectedAccount) : 'https://launcher-files.modrinth.com/assets/steve_head.png'"
    />
    <div v-if="selectedAccount && isElyBy(selectedAccount)" class="elyby-avatar size-36" :style="{ backgroundImage: `url('${getSkinUrl(selectedAccount)}'), url('${getSkinUrl(selectedAccount)}')` }"></div>
    <div class="flex flex-col w-full">
      <span>
        <component v-if="selectedAccount && !isElyBy(selectedAccount)" :is="getAccountType(selectedAccount)" class="vector-icon" />
        <span v-if="selectedAccount && isElyBy(selectedAccount)" class="elyby-badge-small">E</span>
        {{ selectedAccount ? selectedAccount.username : 'Select account' }}
      </span>
      <span class="text-secondary text-xs">{{ selectedAccount && isElyBy(selectedAccount) ? 'Ely.by account' : 'Minecraft account' }}</span>
    </div>
    <DropdownIcon class="w-5 h-5 shrink-0" />
  </div>
  <transition name="fade">
    <Card v-if="showCard || mode === 'isolated'" ref="card" class="account-card"
      :class="{ expanded: mode === 'expanded', isolated: mode === 'isolated' }">
      <div v-if="selectedAccount" class="selected account">
        <Avatar v-if="!isElyBy(selectedAccount)" size="xs" :src="getAvatarUrl(selectedAccount)" />
        <div v-if="isElyBy(selectedAccount)" class="elyby-avatar size-xs" :style="{ backgroundImage: `url('${getSkinUrl(selectedAccount)}'), url('${getSkinUrl(selectedAccount)}')` }"></div>
        <div>
          <h4>
            <component v-if="!isElyBy(selectedAccount)" :is="getAccountType(selectedAccount)" class="vector-icon" />
            <span v-if="isElyBy(selectedAccount)" class="elyby-badge-small">E</span>
            {{ selectedAccount.username }}
          </h4>
          <p>{{ isElyBy(selectedAccount) ? 'Ely.by' : 'Selected' }}</p>
        </div>
        <Button v-tooltip="'Log out'" icon-only color="raised" @click="logout(selectedAccount.id)">
          <TrashIcon />
        </Button>
      </div>
      <div v-else class="login-section account flex flex-col gap-2 p-4 min-w-[240px]">
        <h4 class="text-sm font-bold text-secondary mb-2" style="color: var(--color-text-secondary); opacity: 0.8; margin-bottom: 8px;">Sign in to play</h4>
        <Button
          class="w-full flex items-center justify-start gap-3 py-2 px-3 rounded-xl text-sm font-medium hover:brightness-110 transition-all cursor-pointer"
          style="display: flex; align-items: center; justify-content: flex-start; gap: 10px; width: 100%; border-radius: 8px;"
          @click="login()"
        >
          <MicrosoftIcon class="w-5 h-5 shrink-0" />
          <span>Sign in with Microsoft</span>
        </Button>
        <Button
          class="w-full flex items-center justify-start gap-3 py-2 px-3 rounded-xl text-sm font-medium hover:brightness-110 transition-all cursor-pointer text-white"
          style="display: flex; align-items: center; justify-content: flex-start; gap: 10px; width: 100%; border-radius: 8px; background: linear-gradient(135deg, #1a9e5c, #0e7a44); color: white;"
          @click="elybyLogin()"
        >
          <span class="w-5 h-5 flex items-center justify-center bg-white/20 rounded-full font-bold text-[10px] text-white shrink-0" style="display: inline-flex; align-items: center; justify-content: center; width: 20px; height: 20px; background: rgba(255,255,255,0.2); border-radius: 50%; font-weight: bold; font-size: 10px; color: white;">E</span>
          <span>Sign in with Ely.by</span>
        </Button>
        <Button
          class="w-full flex items-center justify-start gap-3 py-2 px-3 rounded-xl text-sm font-medium hover:brightness-110 transition-all cursor-pointer"
          style="display: flex; align-items: center; justify-content: flex-start; gap: 10px; width: 100%; border-radius: 8px;"
          @click="tryOfflineLogin()"
        >
          <PirateIcon class="w-5 h-5 shrink-0" />
          <span>Play Offline</span>
        </Button>
      </div>
      <div v-if="displayAccounts.length > 0" class="account-group">
        <div v-for="account in displayAccounts" :key="account.id" class="account-row">
          <Button class="option account" @click="setAccount(account)">
            <Avatar v-if="!isElyBy(account)" :src="getAvatarUrl(account)" class="icon" />
            <div v-if="isElyBy(account)" class="elyby-avatar icon" :style="{ backgroundImage: `url('${getSkinUrl(account)}'), url('${getSkinUrl(account)}')` }"></div>
            <p class="account-type">
              <component v-if="!isElyBy(account)" :is="getAccountType(account)" class="vector-icon" />
              <span v-if="isElyBy(account)" class="elyby-badge-small">E</span>
              {{ account.username }}
            </p>
          </Button>
          <Button v-tooltip="'Log out'" icon-only @click="logout(account.id)">
            <TrashIcon />
          </Button>
        </div>
      </div>
      <div v-if="accounts.length > 0" class="login-section account centered flex flex-col gap-2 p-4 min-w-[240px]" style="display: flex; flex-direction: column; align-items: stretch; width: 100%;">
        <h5 class="text-xs font-bold text-secondary mb-1" style="color: var(--color-text-secondary); opacity: 0.8; margin-bottom: 4px; font-size: 11px;">Add Another Account</h5>
        <Button
          class="w-full flex items-center justify-start gap-3 py-2 px-3 rounded-xl text-sm font-medium hover:brightness-110 transition-all cursor-pointer"
          style="display: flex; align-items: center; justify-content: flex-start; gap: 10px; width: 100%; border-radius: 8px;"
          @click="login()"
        >
          <MicrosoftIcon class="w-5 h-5 shrink-0" />
          <span>Sign in with Microsoft</span>
        </Button>
        <Button
          class="w-full flex items-center justify-start gap-3 py-2 px-3 rounded-xl text-sm font-medium hover:brightness-110 transition-all cursor-pointer text-white"
          style="display: flex; align-items: center; justify-content: flex-start; gap: 10px; width: 100%; border-radius: 8px; background: linear-gradient(135deg, #1a9e5c, #0e7a44); color: white;"
          @click="elybyLogin()"
        >
          <span class="w-5 h-5 flex items-center justify-center bg-white/20 rounded-full font-bold text-[10px] text-white shrink-0" style="display: inline-flex; align-items: center; justify-content: center; width: 20px; height: 20px; background: rgba(255,255,255,0.2); border-radius: 50%; font-weight: bold; font-size: 10px; color: white;">E</span>
          <span>Sign in with Ely.by</span>
        </Button>
        <Button
          class="w-full flex items-center justify-start gap-3 py-2 px-3 rounded-xl text-sm font-medium hover:brightness-110 transition-all cursor-pointer"
          style="display: flex; align-items: center; justify-content: flex-start; gap: 10px; width: 100%; border-radius: 8px;"
          @click="tryOfflineLogin()"
        >
          <PirateIcon class="w-5 h-5 shrink-0" />
          <span>Play Offline</span>
        </Button>
      </div>
    </Card>
  </transition>
  <ModalWrapper ref="loginOfflineModal" class="modal" header="Add new offline account">
    <div class="modal-body">
      <div class="label">Enter offline username</div>
      <input type="text" v-model="playerName" placeholder="Provide offline player name" @keyup.enter="offlineLoginFinally()" />
      <Button icon-only color="secondary" @click="offlineLoginFinally()">
        Continue
      </Button>
    </div>
  </ModalWrapper>
  <ModalWrapper ref="loginElyByModal" class="modal" header="Add new Ely.by account">
    <div class="modal-body">
      <div class="label">Enter Ely.by Username</div>
      <input type="text" v-model="elybyPlayerName" placeholder="Provide Ely.by username" @keyup.enter="elybyLoginFinally()" />
      <Button icon-only color="secondary" @click="elybyLoginFinally()">
        Continue
      </Button>
    </div>
  </ModalWrapper>
  <ModalWrapper ref="loginErrorModal" class="modal" header="Error while proceed">
    <div class="modal-body">
      <div class="label">Error occurred while adding offline account</div>
      <Button color="primary" @click="retryOfflineLogin()">
        Try again
      </Button>
    </div>
  </ModalWrapper>
  <ModalWrapper ref="unexpectedErrorModal" class="modal" header="Ошибка">
    <div class="modal-body">
      <div class="label">Unexcepted error</div>
    </div>
  </ModalWrapper>
</template>

<script setup>
import { DropdownIcon, PlusIcon, TrashIcon, LogInIcon, PirateIcon as Offline, MicrosoftIcon as License, MicrosoftIcon, PirateIcon } from '@modrinth/assets'
import { Avatar, Button, Card } from '@modrinth/ui'
import { ref, computed, onMounted, onBeforeUnmount, onUnmounted } from 'vue'
import {
  offline_login,
  users,
  remove_user,
  set_default_user,
  login as login_flow,
  get_default_user,
  elyby_login,
} from '@/helpers/auth'
import { handleError } from '@/store/state.js'
import { trackEvent } from '@/helpers/analytics'
import { process_listener } from '@/helpers/events'
import { handleSevereError } from '@/store/error.js'
import ModalWrapper from './modal/ModalWrapper.vue'

defineProps({
  mode: {
    type: String,
    required: true,
    default: 'normal',
  },
})

const emit = defineEmits(['change'])

const accounts = ref({})
const defaultUser = ref()
const loginOfflineModal = ref(null)
const loginElyByModal = ref(null)
const loginErrorModal = ref(null)
const unexpectedErrorModal = ref(null)
const playerName = ref('')
const elybyPlayerName = ref('')

async function tryOfflineLogin() { // Patched
  loginOfflineModal.value.show()
}

async function offlineLoginFinally() { // Patched
  let name = playerName.value
  if (name.length > 1 && name.length < 20 && name !== '') {
    const loggedIn = await offline_login(name).catch(handleError)
    loginOfflineModal.value.hide()
    if (loggedIn) {
      await setAccount(loggedIn)
      await refreshValues()
    } else {
      unexpectedErrorModal.value.show()
    }
    playerName.value = ''
  } else {
    playerName.value = ''
    loginOfflineModal.value.hide()
    loginErrorModal.value.show()
  }
}

function retryOfflineLogin() { // Patched
  loginErrorModal.value.hide()
  tryOfflineLogin()
}

function isElyBy(account) {
  return account.access_token === 'elyby_offline_token'
}

function getAvatarUrl(account) {
  return `https://mc-heads.net/avatar/${account.username}/128`
}

function getSkinUrl(account) {
  return `http://skinsystem.ely.by/skins/${encodeURIComponent(account.username)}.png`
}

function getAccountType(account) {
  if (isElyBy(account)) {
    return null // handled separately with the E badge
  } else if (account.access_token && account.access_token !== 'null' && account.access_token !== '') {
    return License // Microsoft
  } else {
    return Offline
  }
}

async function refreshValues() {
  defaultUser.value = await get_default_user().catch(handleError)
  accounts.value = await users().catch(handleError)
}
defineExpose({
  refreshValues,
})
await refreshValues()

const displayAccounts = computed(() =>
  accounts.value.filter((account) => defaultUser.value !== account.id),
)

const selectedAccount = computed(() =>
  accounts.value.find((account) => account.id === defaultUser.value),
)

async function setAccount(account) {
  defaultUser.value = account.id
  await set_default_user(account.id).catch(handleError)
  emit('change')
}

async function login() {
  const loggedIn = await login_flow().catch(handleSevereError)

  if (loggedIn) {
    await setAccount(loggedIn)
    await refreshValues()
  }

  trackEvent('AccountLogIn')
}

async function tryElyByLogin() {
  loginElyByModal.value.show()
}

async function elybyLoginFinally() {
  let name = elybyPlayerName.value.trim()
  if (name.length > 0 && name !== '') {
    loginElyByModal.value.hide()
    const loggedIn = await elyby_login(name).catch((err) => {
      handleError(err)
      loginErrorModal.value.show()
      return null
    })
    if (loggedIn) {
      await setAccount(loggedIn)
      await refreshValues()
    }
    elybyPlayerName.value = ''
  } else {
    elybyPlayerName.value = ''
    loginElyByModal.value.hide()
    loginErrorModal.value.show()
  }
}

async function elybyLogin() {
  await tryElyByLogin()
  trackEvent('AccountLogInElyBy')
}

const logout = async (id) => {
  await remove_user(id).catch(handleError)
  await refreshValues()
  if (!selectedAccount.value && accounts.value.length > 0) {
    await setAccount(accounts.value[0])
    await refreshValues()
  } else {
    emit('change')
  }
  trackEvent('AccountLogOut')
}

const showCard = ref(false)
const card = ref(null)
const button = ref(null)
const handleClickOutside = (event) => {
  const elements = document.elementsFromPoint(event.clientX, event.clientY)
  if (
    card.value &&
    card.value.$el !== event.target &&
    !elements.includes(card.value.$el) &&
    !button.value.contains(event.target)
  ) {
    toggleMenu(false)
  }
}

function toggleMenu(override = true) {
  if (showCard.value || !override) {
    showCard.value = false
  } else {
    showCard.value = true
  }
}

const unlisten = await process_listener(async (e) => {
  if (e.event === 'launched') {
    await refreshValues()
  }
})

onMounted(() => {
  window.addEventListener('click', handleClickOutside)
})

onBeforeUnmount(() => {
  window.removeEventListener('click', handleClickOutside)
})

onUnmounted(() => {
  unlisten()
})
</script>

<style scoped lang="scss">
.selected {
  background: var(--color-brand-highlight);
  border-radius: var(--radius-lg);
  color: var(--color-contrast);
  gap: 1rem;
}

.login-section {
  background: var(--color-bg);
  border-radius: var(--radius-lg);
  gap: 1rem;
}


.vector-icon {
  width: 12px;
  height: 12px;
}

.elyby-badge-small {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 12px;
  height: 12px;
  background: linear-gradient(135deg, #1a9e5c, #0e7a44);
  border-radius: 50%;
  font-size: 7px;
  font-weight: bold;
  color: white;
  vertical-align: middle;
  margin-right: 2px;
}

.elyby-avatar {
  background-repeat: no-repeat, no-repeat;
  image-rendering: pixelated;
  border-radius: var(--radius-md);
  background-color: var(--color-raised-bg);
  flex-shrink: 0;
  
  --avatar-size: 36px;
  width: var(--avatar-size);
  height: var(--avatar-size);
  background-size: calc(var(--avatar-size) * 8) auto, calc(var(--avatar-size) * 8) auto;
  background-position: calc(var(--avatar-size) * -5) calc(var(--avatar-size) * -1), calc(var(--avatar-size) * -1) calc(var(--avatar-size) * -1);
}
.elyby-avatar.size-36 { --avatar-size: 36px; }
.elyby-avatar.size-xs { --avatar-size: 20px; }
.elyby-avatar.icon { --avatar-size: 24px; margin-right: 0.5rem; }

.account {
  width: max-content;
  display: flex;
  align-items: center;
  text-align: left;
  padding: 0.5rem 1rem;

  h4,
  p {
    margin: 0;
  }
}

.account-card {
  position: fixed;
  display: flex;
  flex-direction: column;
  margin-top: 0.5rem;
  right: 2rem;
  z-index: 11;
  gap: 0.5rem;
  padding: 1rem;
  border: 1px solid var(--color-button-bg);
  width: max-content;
  user-select: none;
  -ms-user-select: none;
  -webkit-user-select: none;
  max-height: 98vh;
  overflow-y: auto;

  &::-webkit-scrollbar-track {
    border-top-right-radius: 1rem;
    border-bottom-right-radius: 1rem;
  }

  &::-webkit-scrollbar {
    border-top-right-radius: 1rem;
    border-bottom-right-radius: 1rem;
  }

  &.hidden {
    display: none;
  }

  &.expanded {
    left: 13.5rem;
  }

  &.isolated {
    position: relative;
    left: 0;
    top: 0;
  }
}

.accounts-title {
  font-size: 1.2rem;
  font-weight: bolder;
}

.centered {
  display: flex;
  gap: 1rem;
  margin: auto;
}

.account-group {
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.option {
  width: calc(100% - 2.25rem);
  background: var(--color-raised-bg);
  color: var(--color-base);
  box-shadow: none;

  img {
    margin-right: 0.5rem;
  }
}

.icon {
  --size: 1.5rem !important;
}

.account-row {
  display: flex;
  flex-direction: row;
  gap: 0.5rem;
  vertical-align: center;
  justify-content: space-between;
  padding-right: 1rem;
}

.fade-enter-active,
.fade-leave-active {
  transition:
    opacity 0.25s ease,
    translate 0.25s ease,
    scale 0.25s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
  translate: 0 -2rem;
  scale: 0.9;
}

.avatar-button {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  color: var(--color-base);
  background-color: var(--color-button-bg);
  border-radius: var(--radius-md);
  width: 100%;
  padding: 0.5rem 0.75rem;
  text-align: left;

  &.expanded {
    border: 1px solid var(--color-button-bg);
    padding: 1rem;
  }
}

.avatar-text {
  margin: auto 0 auto 0.25rem;
  display: flex;
  flex-direction: column;
}

.text {
  width: 6rem;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.accounts-text {
  display: flex;
  align-items: center;
  gap: 0.25rem;
  margin: 0;
}

.qr-code {
  background-color: white !important;
  border-radius: var(--radius-md);
}

.modal-body {
  display: flex;
  flex-direction: row;
  gap: var(--gap-lg);
  align-items: center;
  padding: var(--gap-xl);

  .modal-text {
    display: flex;
    flex-direction: column;
    gap: var(--gap-sm);
    width: 100%;

    h2,
    p {
      margin: 0;
    }

    .code-text {
      display: flex;
      flex-direction: row;
      gap: var(--gap-xs);
      align-items: center;

      .code {
        background-color: var(--color-bg);
        border-radius: var(--radius-md);
        border: solid 1px var(--color-button-bg);
        font-family: var(--mono-font);
        letter-spacing: var(--gap-md);
        color: var(--color-contrast);
        font-size: 2rem;
        font-weight: bold;
        padding: var(--gap-sm) 0 var(--gap-sm) var(--gap-md);
      }

      .btn {
        width: 2.5rem;
        height: 2.5rem;
      }
    }
  }
}

.button-row {
  display: flex;
  flex-direction: row;
}

.modal {
  position: absolute;
}

.code {
  color: var(--color-brand);
  padding: 0.05rem 0.1rem;
  // row not column
  display: flex;

  .card {
    background: var(--color-base);
    color: var(--color-contrast);
    padding: 0.5rem 1rem;
  }
}
</style>
