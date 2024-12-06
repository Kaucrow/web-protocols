<script lang="ts">
  import type { IFile } from "@/types";
  import { fade, slide } from 'svelte/transition';
  import { 
    sendFile, 
    deleteFile, 
    changeWorkingDirectory, 
    changeLocal,
    downloadServerFile

  } from '../Utils/fetch';
  import Modal from "./Modal.svelte";



  let {title,files = [],isRemote = false , reloadFiles, showToast} = $props()

  let showFileModal = $state(false);
  let selectedFile: IFile | null = $state(null);

  async function handleFileClick(file: IFile) {
    selectedFile = file;
    console.log(selectedFile)
    showFileModal = true;
  }

  async function handleFileAction(action: 'send' | 'delete' | 'change' | 'Download') {
    if (!selectedFile) return;
    let nameAction

    const actions= {
      send:async()=>{ 
        if (!selectedFile) return;
        nameAction = 'sended'
        return await sendFile(selectedFile.name)
      },
      delete: async()=>{
        if (!selectedFile) return;
        nameAction = 'deleted'
        return await deleteFile(selectedFile.name)
      },
      Download: async()=>{
        if (!selectedFile) return;
         nameAction = 'downloaded'
         return await downloadServerFile(selectedFile.name)
      },
      change: async()=>{
        if (isRemote) {
          if (!selectedFile) return;
          await changeWorkingDirectory(selectedFile.name)
        } else {
          if (!selectedFile) return;
          await changeLocal(selectedFile.name)
        }
      },
    }

    try {
      const result =await actions[action]();
      console.log(result)
      if (result.success) {
        showToast(`File ${nameAction} Successfully`, 'success');
        console.log('Action Successful',action,result.message);
        await reloadFiles();
      }
    } catch (error) {
      console.error(error);
    }



    showFileModal = false;
    selectedFile = null;
  }
</script>

<style>
  .scrollable-content {
    max-height: 69vh;
    overflow-y: auto;
    padding-right: 15px;
    margin-right: -1px;
  }
  .scrollable-content::-webkit-scrollbar {
    width: 3px;
  }

  .scrollable-content::-webkit-scrollbar-track {
    background: rgba(255, 255, 255, 0.05);
  }

  .scrollable-content::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.1);
    border-radius: 3px;
  }

  .file-row {
    transition: background-color 0.2s ease;
  }
</style>

<div class="bg-dark-secondary p-4 rounded-lg shadow-lg h-svh scrollable-content" transition:fade>
  <h2 class="text-xl font-bold mb-4">{title}</h2>
  <div class="overflow-auto max-h-[calc(100vh-300px)] scrollable-content">
    <table class="w-full">
      <thead>
        <tr class="text-left border-b border-gray-600">
          <th class="py-2">Name</th>
          <th class="py-2">Size</th>
          <th class="py-2">Modified</th>
        </tr>
      </thead>
      <tbody>
        {#each files as file (file.name)}
          <tr
            class="file-row hover:bg-dark-accent cursor-pointer"
            ondblclick={() => handleFileClick(file)}
            transition:slide
          >
            <td class="py-2 flex items-center">
              <span class="mr-2">
                {#if file.type === 'directory'}
                  📁
                {:else}
                  📄
                {/if}
              </span>
              {file.name}
            </td>
            <td class="py-2">{file.size || '-'}</td>
            <td class="py-2">{file.date || '-'}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
</div>

{#if selectedFile}
  <Modal
    show={showFileModal}
    title={isRemote ? "Remote File Actions" : "Local File Actions"}
  >
    <div class="text-center mb-4">
      <p class="text-lg">{selectedFile.name}</p>
    </div>
    
    <div class="flex justify-center gap-4">
      {#if selectedFile.type === 'directory'}
        <button
          class="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded transition-colors"
          onclick={() => handleFileAction('change')}
        >
          Open Directory
        </button>
      {:else}
        {#if isRemote}
          <button
            class="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded transition-colors"
            onclick={() => handleFileAction('Download')}
          >
            Download
          </button>
          <button
            class="px-4 py-2 bg-red-600 hover:bg-red-700 rounded transition-colors"
            onclick={() => handleFileAction('delete')}
          >
            Delete
          </button>
        {:else}
          <button
            class="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded transition-colors"
            onclick={() => handleFileAction('send')}
          >
            Send to Server
          </button>
        {/if}
      {/if}
      <button
        class="px-4 py-2 bg-gray-600 hover:bg-gray-700 rounded transition-colors"
        onclick={() => { showFileModal = false; selectedFile = null; }}
      >
        Exit
      </button>
    </div>
  </Modal>
{/if}