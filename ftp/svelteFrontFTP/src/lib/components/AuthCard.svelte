<script lang="ts">
  import { connect, disconnect } from "../Utils/fetch";



  let host = $state('');
  let username = $state('');
  let password = $state('');
  let passive = $state(false);
  let isConnected = $state(false);

  let { reloadFiles } = $props();

  async function handleConnection() {
    isConnected = !isConnected;
    if (isConnected) {
      console.log('Connecting...');
      const { success, message} = await connect(host, username, password);
      if (!success) {
        isConnected = false;
      }
      console.log(message);
      await reloadFiles();
      
    } 
    if (!isConnected) {
      console.log('Disconnecting...');
      host = '';
      username = '';
      password = '';
      passive = false;
      const  { success, message} = await disconnect();
      if (success) {
        console.log(message);
        reloadFiles();

      }
    }

  }

</script>

<div class="bg-dark-secondary p-4 rounded-lg shadow-lg w-full">
  <h2 class="text-xl font-bold mb-4">FTP Connection</h2>
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
      {isConnected ? 'Disconnect' : 'Connect'}
    </button>
  </div>
</div>

<style>
  .dot {
    transition: transform 0.3s ease-in-out;
  }
</style>