<!-- NOTE: this should be treated as a singleton, backend support currently only works for one instance of this component -->
<script lang="ts">
    import { listen, type UnlistenFn } from "@tauri-apps/api/event";
    import { onMount, onDestroy } from "svelte";
    import { box } from "$lib/state.svelte";

    let canvas: HTMLCanvasElement;
    let ctx: CanvasRenderingContext2D;

    const { width, height } = $props();

    let unlisten: UnlistenFn | null = null;

    const pixelBuffer = new Uint8ClampedArray(width * height * 4);
    const imageData = new ImageData(pixelBuffer, width, height);

    export function getImageData(): ImageData {
        return imageData;
    }

    let busy = false;

    onMount(async () => {
        ctx = canvas.getContext("2d")!;

        unlisten = await listen<string>("frame", (event) => {
            if (busy || box.freeze) return;
            busy = true;

            const raw = atob(event.payload);

            for (let i = 0; i < raw.length; i++) {
                pixelBuffer[i] = raw.charCodeAt(i);
            }

            ctx.putImageData(imageData, 0, 0);

            busy = false;
        });
    });

    onDestroy(async () => {
        if (unlisten) {
            unlisten();
            unlisten = null;
        }

        box.freeze = false;
    });
</script>

<canvas bind:this={canvas} {width} {height}></canvas>
