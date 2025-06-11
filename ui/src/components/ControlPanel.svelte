<script lang="ts">
    import { Button } from "$lib/components/ui/button/index.js";
    import {get, post} from "$lib/api";
    let { config = $bindable() } = $props();
    function shutdown() {
        // post shutdown: true to the server
        post({shutdown: true}).then(() => {
            // TODO: Find what to do after shutdown
        });
    }

    function pause() {
        // post paused: true to the server
        post({paused: true}).then((d) => {
            get().then(data => {
                config = data;
            })
        });
        config.paused = true;
    }

    function resume() {
        // post paused: false to the server
        post({paused: false}).then((d) => {
            get().then(data => {
                config = data;
            })
        });
        config.paused = false;
    }
</script>

{#if config.paused}
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