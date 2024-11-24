<script lang="ts">
  import type { IFile } from "@/types";
  export let title: string;
  export let files: IFile[]= [];

  function handleFileClick(file: typeof files[0]) {
    // Implement file click logic
  }
</script>

<div class="bg-dark-secondary p-4 rounded-lg shadow-lg h-full">
  <h2 class="text-xl font-bold mb-4">{title}</h2>
  <div class="overflow-auto max-h-[calc(100vh-300px)]">
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