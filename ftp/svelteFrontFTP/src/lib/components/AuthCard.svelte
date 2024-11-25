<script lang="ts">
  import { connect, disconnect } from "../Utils/fetch";
  import Modal from "./Modal.svelte";
  import { slide } from 'svelte/transition';

  let host = $state('');
  let username = $state('');
  let password = $state('');
  let passive = $state(false);
  let isConnected = $state(false);
  let showModal = $state(true);

  let { reloadFiles } = $props();

  async function handleConnection() {
    try {
      if (!isConnected) {
        const { success } = await connect(host, username, password);
        if (success) {
          isConnected = true;
          showModal = false;
          await reloadFiles();
        }
      } else {
        await disconnect();
        
          isConnected = false;
          showModal = true;
          host = '';
          username = '';
          password = '';
          passive = false;
          await reloadFiles(true);
        
      }
    } catch (error) {
      console.error(error);
    }
  }
</script>

{#if isConnected}
<button
onclick={handleConnection}
class="px-4 py-3 bg-red-800 hover:bg-red-700 text-white rounded transition-colors block"
transition:slide
>
Disconnect
</button>
{:else}
  <Modal show={showModal} title="FTP Connection">
    <div class="space-y-4">
      <div>
        <label class="block text-sm mb-1" for="host">Host</label>
        <input
          type="text"
          id="host"
          bind:value={host}
          class="w-full bg-dark-accent text-white p-2 rounded border border-gray-600 focus:border-blue-500 focus:outline-none"
        />
      </div>
      
      <div>
        <label class="block text-sm mb-1" for="username">Username</label>
        <input
          type="text"
          id="username"
          bind:value={username}
          class="w-full bg-dark-accent text-white p-2 rounded border border-gray-600 focus:border-blue-500 focus:outline-none"
        />
      </div>
      
      <div>
        <label class="block text-sm mb-1" for="password">Password</label>
        <input
          type="password"
          id="password"
          bind:value={password}
          class="w-full bg-dark-accent text-white p-2 rounded border border-gray-600 focus:border-blue-500 focus:outline-none"
        />
      </div>
      
      <div class="flex items-center">
        <label class="flex items-center cursor-pointer">
          <div class="relative">
            <input
              type="checkbox"
              bind:checked={passive}
              class="sr-only"
            />
            <div class="w-10 h-6 bg-dark-accent rounded-full shadow-inner"></div>
            <div class="dot absolute w-4 h-4 bg-white rounded-full transition transform {passive ? 'translate-x-5' : 'translate-x-1'} top-1"></div>
          </div>
          <div class="ml-3 text-sm">Passive Mode</div>
        </label>
      </div>
      
      <button
        onclick={handleConnection}
        class="w-full py-2 px-4 rounded bg-blue-600 hover:bg-blue-700 text-white transition-colors"
      >
        Connect
      </button>
    </div>
  </Modal>
{/if}

<style>
  .dot {
    transition: transform 0.3s ease-in-out;
  }
</style>