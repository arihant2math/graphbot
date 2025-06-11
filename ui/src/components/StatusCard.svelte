<script lang="ts">
    import * as Card from "$lib/components/ui/card/index.js";
    import ControlPanel from "./ControlPanel.svelte";
    let { config = $bindable() } = $props();
</script>

<Card.Root class="w-full max-w-sm">
    <Card.Header>
        <Card.Title>{config.username}</Card.Title>
        <Card.Description>Searching on <a href="{config.wiki}wiki/{config.search_category}" class="ml-auto inline-block text-sm underline-offset-4 hover:underline">{config.search_category}</a> on <a href="{config.wiki}" class="ml-auto inline-block text-sm underline-offset-4 hover:underline">{config.wiki}</a></Card.Description>
    </Card.Header>
    <Card.Content>
<!--        Dual column -->
        <div class="grid grid-cols-2 gap-4">
            <div>
                <h3 class="text-lg font-semibold">Status</h3>
                <p>{config.paused ? "Paused" : "Active"}</p>
            </div>
            <div>
                <h3 class="text-lg font-semibold">RPC</h3>
                <p><b>Port</b> {config.rpc.port}</p>
                <p><b>Path</b> {config.rpc.path}</p>
            </div>
        </div>
    </Card.Content>
    <Card.Footer class="flex-row gap-2">
        <ControlPanel bind:config={config}/>
    </Card.Footer>
</Card.Root>
