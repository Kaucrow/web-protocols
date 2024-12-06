<script lang="ts">
  import AuthCard from './lib/components/AuthCard.svelte';
  import FileExplorer from './lib/components/FileExplorer.svelte';
  import type { IFile } from './types';
  import { disconnect, getClientFiles, getServerFiles } from './lib/Utils/fetch';
  import { onMount } from 'svelte';
  import { toasts, ToastContainer, FlatToast, BootstrapToast }  from "svelte-toasts";

  let localFiles: IFile[] = $state([]);
  let remoteFiles: IFile[] = $state([]);

  const reloadFiles = async(erase = false) => {
    console.log(erase)
    if (erase) {
      localFiles = []
      remoteFiles = []
      return
      
    }
    const [newLocalFiles, newRemoteFiles] = await Promise.all([
      getClientFiles(),
      getServerFiles()
    ]);

    localFiles = [...newLocalFiles];
    remoteFiles = [{ name: '..', type: 'directory' }, ...newRemoteFiles]
  };

  onMount(async () => {
    disconnect();
    localFiles = []
    remoteFiles = []

  });

  const showToast = (title:string,type:any) => {
    toasts.add({
      title: title,
      duration: 5000, // 0 or negative to avoid auto-remove
      placement: 'bottom-right',
      type: type,
    });
  };


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
  <h1 class="text-4xl font-bold text-center mb-6">FTP SERVER</h1>
  <div class="grid grid-cols-2 gap-6">
    <!-- Left column -->
    <div class="space-y-6">

      <FileExplorer title="Local Files" files={localFiles} reloadFiles={reloadFiles} showToast={showToast} />
    </div>
    <ToastContainer placement="bottom-right" let:data={data}>
      <FlatToast {data} /> <!-- Provider template for your toasts -->
    </ToastContainer>
    <!-- Right column -->
    <div class="space-y-6">
      <FileExplorer title="Remote Files" files={remoteFiles} isRemote={true}  reloadFiles={reloadFiles} showToast={showToast}/>
    </div>

      <AuthCard reloadFiles={reloadFiles} />

  </div>
</div>