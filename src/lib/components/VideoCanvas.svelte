<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { listen, type UnlistenFn } from "@tauri-apps/api/event";
    import { onMount, onDestroy } from "svelte";

    let canvas: HTMLCanvasElement;
    let ctx: CanvasRenderingContext2D;

    const { width, height } = $props();

    let unlisten: UnlistenFn | null = null;

    const pixelBuffer = new Uint8ClampedArray(width * height * 4);
    const imageData = new ImageData(pixelBuffer, width, height);

    let busy = false;

    onMount(async () => {
        ctx = canvas.getContext("2d")!;

        unlisten = await listen<string>("frame", (event) => {
            if (busy) return;
            busy = true;

            const raw = atob(event.payload);

            for (let i = 0; i < raw.length; i++) {
                pixelBuffer[i] = raw.charCodeAt(i);
            }

            ctx.putImageData(imageData, 0, 0);

            busy = false;
        });

        await invoke("start_stream");
    });

    onDestroy(async () => {
        if (unlisten) {
            unlisten();
            unlisten = null;
        }

        await invoke("stop_stream");
    });
</script>

<canvas bind:this={canvas} {width} {height}></canvas>
