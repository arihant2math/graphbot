<script lang="ts">
    import StatusCard from "../components/StatusCard.svelte";
    import {onMount} from "svelte";
    import {get} from "$lib/api";

    let config = $state(null);
    let error = $state(null);
    // get config from the server
    onMount(() => {
        get().then(data => {
                config = data;
            })
            .catch(error => {
                console.error("Error fetching config:", error);
                error = error;
            });
    });
</script>
<div class="m-4">
    {#if config !== null}
        <StatusCard bind:config={config}/>
    {:else if error !== null}
        <p>Error fetching configuration: {error}</p>
    {:else}
        <p>Loading configuration ...</p>
    {/if}
</div>
