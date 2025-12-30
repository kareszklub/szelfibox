<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { listen } from "@tauri-apps/api/event";
    import { onMount } from "svelte";

    let canvas: HTMLCanvasElement | null = null;
    let ctx: CanvasRenderingContext2D | null = null;

    const { width, height } = $props();

    onMount(async () => {
        ctx = canvas!.getContext("2d");

        listen("frame", (event) => {
            const raw = atob(event.payload as string);
            const arr = new Uint8ClampedArray(raw.length);
            for (let i = 0; i < raw.length; i++) arr[i] = raw.charCodeAt(i);
            const img = new ImageData(arr, width, height);
            ctx!.putImageData(img, 0, 0);
        });

        invoke("start_stream");
    });
</script>

<canvas bind:this={canvas} width="640" height="480"></canvas>
