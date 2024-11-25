<script lang="ts">
  import AuthCard from './lib/components/AuthCard.svelte';
  import FileExplorer from './lib/components/FileExplorer.svelte';
  import ActionButtons from './lib/components/ActionButtons.svelte';
  import type { IFile } from './types';
  import {getClientFiles, getServerFiles} from './lib/Utils/fetch';
  import { onMount } from 'svelte';
  // Sample data :>


  let localFiles: IFile[] = $state([
    { name: 'so', type: 'directory', date: '2024-03-10' },
    { name: 'd', type: 'file', size: '1.2 MB', date: '2024-03-09' },
    { name: 'f', type: 'file', size: '4 KB', date: '2024-03-08' },
    { name: 'so', type: 'directory', date: '2024-03-10' },
    { name: 'd', type: 'file', size: '1.2 MB', date: '2024-03-09' },
    { name: 'f', type: 'file', size: '4 KB', date: '2024-03-08' },
    { name: 'so', type: 'directory', date: '2024-03-10' },
    { name: 'd', type: 'file', size: '1.2 MB', date: '2024-03-09' },
    { name: 'f', type: 'file', size: '4 KB', date: '2024-03-08' },
    { name: 'so', type: 'directory', date: '2024-03-10' },
    { name: 'd', type: 'file', size: '1.2 MB', date: '2024-03-09' },
    { name: 'so', type: 'directory', date: '2024-03-10' },
    { name: 'd', type: 'file', size: '1.2 MB', date: '2024-03-09' },
    { name: 'f', type: 'file', size: '4 KB', date: '2024-03-08' },
    { name: 'so', type: 'directory', date: '2024-03-10' },
    { name: 'd', type: 'file', size: '1.2 MB', date: '2024-03-09' },
    { name: 'so', type: 'directory', date: '2024-03-10' },
    { name: 'd', type: 'file', size: '1.2 MB', date: '2024-03-09' },
    { name: 'f', type: 'file', size: '4 KB', date: '2024-03-08' },
    { name: 'so', type: 'directory', date: '2024-03-10' },
    { name: 'd', type: 'file', size: '1.2 MB', date: '2024-03-09' },

  ]);

  let remoteFiles: IFile[] = $state([
    { name: 'so', type: 'directory', date: '2024-03-10' },
    { name: 'd', type: 'file', size: '1.2 MB', date: '2024-03-09' },
    { name: 'f', type: 'file', size: '4 KB', date: '2024-03-08' },
    { name: 'so', type: 'directory', date: '2024-03-10' },
    { name: 'd', type: 'file', size: '1.2 MB', date: '2024-03-09' },
    { name: 'f', type: 'file', size: '4 KB', date: '2024-03-08' },
    { name: 'so', type: 'directory', date: '2024-03-10' },
    { name: 'd', type: 'file', size: '1.2 MB', date: '2024-03-09' },
    { name: 'f', type: 'file', size: '4 KB', date: '2024-03-08' },
    { name: 'so', type: 'directory', date: '2024-03-10' },
    { name: 'd', type: 'file', size: '1.2 MB', date: '2024-03-09' },
    { name: 'so', type: 'directory', date: '2024-03-10' },
    { name: 'd', type: 'file', size: '1.2 MB', date: '2024-03-09' },
    { name: 'f', type: 'file', size: '4 KB', date: '2024-03-08' },
    { name: 'so', type: 'directory', date: '2024-03-10' },
    { name: 'd', type: 'file', size: '1.2 MB', date: '2024-03-09' },
    { name: 'so', type: 'directory', date: '2024-03-10' },
    { name: 'd', type: 'file', size: '1.2 MB', date: '2024-03-09' },
    { name: 'f', type: 'file', size: '4 KB', date: '2024-03-08' },
    { name: 'so', type: 'directory', date: '2024-03-10' },
    { name: 'd', type: 'file', size: '1.2 MB', date: '2024-03-09' },
  ]);

  const reloadFiles =async(server=[]) => {
    if(server.length===0){
      remoteFiles = server;
    }
    remoteFiles = await getServerFiles();

  };

  onMount(async () => {
    console.log('Fetching files onMount...');
    
    localFiles= await getClientFiles();

  });

</script>

<style>
  :global(body) {
    overflow: hidden;
  }

  .scrollable-content {
    max-height: 69vh;
    overflow-y: auto;
    padding-right: 15px;
    margin-right: -1px;
  }
  .scrollable-content::-webkit-scrollbar {
    width: 1px;
  }

  .scrollable-content::-webkit-scrollbar-track {
    background: rgba(255, 255, 255, 0.05);
  }

  .scrollable-content::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.1);
    border-radius: px;
  }
</style>

<div class="min-h-screen p-6 bg-dark-primary scrollable-content">
  <div class="grid grid-cols-2 gap-6">
    <!-- Left column -->
    <div class="space-y-6">
      <AuthCard reloadFiles={reloadFiles} />
      <FileExplorer title="Local Files" files={localFiles} />
    </div>
    
    <!-- Right column -->
    <div class="space-y-6">
      <ActionButtons />
      <FileExplorer title="Remote Files" files={remoteFiles} />
    </div>
  </div>
</div>