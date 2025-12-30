<script lang="ts">
    import VideoCanvas from "$lib/components/VideoCanvas.svelte";
    import { box } from "$lib/state.svelte";
    import imageData from "$lib/components/VideoCanvas.svelte";

    let countdown: number | null = $state(null);
    let videoCanvas: VideoCanvas | null = $state(null);

    const onKeyDown = (e: KeyboardEvent) => {
        if (e.key === "a") {
            onLeftButtonDown();
        } else if (e.key === "d") {
            onRightButtonDown();
        }
    };

    const onLeftButtonDown = () => {
        if (countdown) return;
        console.log("[Left] button pressed");
        if (box.stage === 1) {
            box.stage = 2;
        } else if (box.stage === 2) {
            box.freeze = false;
            countdown = 3;
            let clear = setInterval(() => {
                if (!countdown) return;
                countdown--;
                if (countdown === 0) {
                    countdown = null;
                    clearInterval(clear);

                    console.log("Csinálom a képet, csíz!");
                    box.freeze = true;
                    box.imageData = videoCanvas!.getImageData();
                }
            }, 1000);
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
        if (countdown) return;
        console.log("[Right] button pressed");
        if (box.stage === 1) {
            box.stage = 2;
        } else if (box.stage === 2) {
            // TODO: handle this in a more human way
            if (!box.imageData) {
                console.error("Először csinálj egy képet!");
                return;
            }
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
            <div class="relative">
                {#if countdown}
                    <div
                        class="absolute inset-0 z-10 text-white text-8xl drop-shadow-lg flex justify-center"
                    >
                        <p>{countdown}</p>
                    </div>
                {/if}
                <div class="relative">
                    <VideoCanvas
                        bind:this={videoCanvas}
                        width={640}
                        height={480}
                    />
                </div>
            </div>
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
