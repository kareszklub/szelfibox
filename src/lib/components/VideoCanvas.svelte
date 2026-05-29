<script lang="ts">
    import { listen, type UnlistenFn } from "@tauri-apps/api/event";
    import { invoke } from "@tauri-apps/api/core"; // Tauri v2 core import
    import { onMount, onDestroy } from "svelte";
    import { box } from "$lib/state.svelte";

    let canvas: HTMLCanvasElement;
    let ctx: CanvasRenderingContext2D;

    const { width, height } = $props();

    let unlisten: UnlistenFn | null = null;

    // Keep the single buffer and ImageData object for reuse
    const pixelBuffer = new Uint8ClampedArray(width * height * 4);
    const imageData = new ImageData(pixelBuffer, width, height);

    export function getImageData(): ImageData {
        return imageData;
    }

    let busy = false;

    onMount(async () => {
        ctx = canvas.getContext("2d", {
            alpha: false, // Optimization: disable alpha if your camera doesn't use it
            desynchronized: true, // Optimization: hints the browser to reduce latency
        })!;

        // We listen for the signal, then fetch the raw bytes
        unlisten = await listen("new-frame-ready", async () => {
            if (busy || box.freeze) return;
            busy = true;

            try {
                // This 'invoke' now receives a raw ArrayBuffer directly
                const buffer = await invoke<ArrayBuffer>("fetch_frame");

                if (buffer instanceof ArrayBuffer) {
                    // Fast memory-to-memory copy (massively faster than a loop)
                    pixelBuffer.set(new Uint8Array(buffer));

                    // Render the updated buffer to the canvas
                    ctx.putImageData(imageData, 0, 0);
                }
            } catch (err) {
                console.error("Binary frame fetch failed:", err);
            } finally {
                busy = false;
            }
        });
    });

    onDestroy(async () => {
        if (unlisten) {
            unlisten();
            unlisten = null;
        }
    });
</script>

<div style="position: relative; display: inline-block;">
    <canvas bind:this={canvas} {width} {height}></canvas>

    <img src="./overlay.png" alt="overlay" />
</div>

<style>
    canvas {
        width: 100%;
        max-width: 800px;
        image-rendering: pixelated;
        background: #000;
        /* transform: scaleX(-1); */
    }
    img {
        position: absolute;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        opacity: 0.7;
        pointer-events: none;
        object-fit: fill;
    }
</style>
