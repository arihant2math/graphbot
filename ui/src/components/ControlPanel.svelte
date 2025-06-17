<script lang="ts">
    import { Button } from "$lib/components/ui/button/index.js";
    import {get, post} from "$lib/api";
    let { config = $bindable() } = $props();
    function shutdown() {
        // post shutdown: true to the server
        post({shutdown_graph_task: true}).then(() => {
            // TODO: Find what to do after shutdown
        });
    }

    function pause() {
        // post paused: true to the server
        post({pause_graph_task: true}).then((d) => {
            get().then(data => {
                config = data;
            })
        });
        config.pause_graph_task = true;
    }

    function resume() {
        // post paused: false to the server
        post({pause_graph_task: false}).then((d) => {
            get().then(data => {
                config = data;
            })
        });
        config.pause_graph_task = false;
    }
</script>

{#if config.pause_graph_task}
    <Button onclick={resume}>
        Resume
    </Button>
{:else}
    <Button onclick={pause}>
        Pause
    </Button>
{/if}
<Button variant="destructive" onclick={shutdown}>
    Shutdown
</Button>