<script lang="ts">
    import VideoCanvas from "../lib/components/VideoCanvas.svelte";
    import { box } from "../lib/state.svelte";

    const onKeyDown = (e: KeyboardEvent) => {
        if (e.key === "a") {
            onLeftButtonDown();
        } else if (e.key === "d") {
            onRightButtonDown();
        }
    };

    const onLeftButtonDown = () => {
        console.log("[Left] button pressed");
        if (box.stage === 1) {
            box.stage = 2;
        } else if (box.stage === 2) {
            console.log("Csinálom a képet, csíz!");
        } else if (box.stage === 3) {
            box.stage = 4;
        } else if (box.stage === 4) {
            console.log("Indítom a nyomtatást");
            box.stage = 1;
        } else {
            throw new Error("Unreachable");
        }
    };

    const onRightButtonDown = () => {
        console.log("[Right] button pressed");
        if (box.stage === 1) {
            box.stage = 2;
        } else if (box.stage === 2) {
            box.stage = 3;
        } else if (box.stage === 3) {
            box.stage = 4;
        } else if (box.stage === 4) {
            box.stage = 1;
        } else {
            throw new Error("Unreachable");
        }
    };
</script>

<svelte:window on:keydown={onKeyDown} />
{#if box.stage === 1}
    <div class="flex justify-center">
        <h1 class="text-3xl">A kezdéshez nyomd meg az egyik gombot</h1>
    </div>
{:else}
    <div class="flex">
        <div class="flex-1 flex justify-center">
            <VideoCanvas width={640} height={480} />
        </div>
        <div class="flex-1">
            {#if box.stage === 2}
                <p>Bal gomb: új kép Jobb gomb: kész :)</p>
            {:else if box.stage === 3}
                <p>Itt egy QR kód, Mindkét gomb: tovább</p>
            {:else}
                <p>Bal gomb: nyomtatás, Jobb gomb: nem kérem</p>
            {/if}
        </div>
    </div>
{/if}
