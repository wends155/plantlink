<script>
  import { SvelteFlowProvider } from "@xyflow/svelte";
  import InnerFlowEditor from "./InnerFlowEditor.svelte";
</script>

<div class="h-screen w-screen">
  <SvelteFlowProvider>
    <InnerFlowEditor />
  </SvelteFlowProvider>
</div>
