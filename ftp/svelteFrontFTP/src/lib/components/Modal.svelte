<script lang="ts">


  let { show ,title,children} = $props();
  
  function closeModal() {
    show = false;
  }
</script>


<div>
    {#if show}
      <div class="fixed inset-0 bg-black bg-opacity-50 z-50 flex items-center justify-center"
      role="dialog">
      <div class="bg-dark-secondary p-6 rounded-lg shadow-xl max-w-md w-full mx-4">
        <h2 class="text-xl font-bold mb-4">{title}</h2>
        <div class="space-y-4">
          {@render children?.()}
        </div>

      </div>
      </div> <!-- Close the div here -->
    {/if} <!-- Close the if block here -->
</div>
