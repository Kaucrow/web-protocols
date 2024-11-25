<script lang="ts">
  import type { IFile } from "@/types";
  export let title: string;
  export let files: IFile[]= [];

  function handleFileClick(file: typeof files[0]) {
    console.log(file.name);
  }
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
    width: 3px;
  }

  .scrollable-content::-webkit-scrollbar-track {
    background: rgba(255, 255, 255, 0.05);
  }

  .scrollable-content::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.1);
    border-radius: px;
  }
</style>

<div class="bg-dark-secondary p-4 rounded-lg shadow-lg h-svh scrollable-content">
  <h2 class="text-xl font-bold mb-4 ">{title}</h2>
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
        {#each files as file}
          <tr
            class="hover:bg-dark-accent cursor-pointer"
            on:click={() => handleFileClick(file)}
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