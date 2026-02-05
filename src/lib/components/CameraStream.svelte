<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import { listen } from "@tauri-apps/api/event";
    import { invoke } from "@tauri-apps/api/core"; // or @tauri-apps/api/tauri in v1
    import JMuxer from "jmuxer";

    let videoElement: HTMLVideoElement;
    let jmuxer: any;

    /**
     * Captures the current frame and returns both raw ImageData and a displayable Blob URL.
     */
    export async function getImageData(): Promise<{
        imageData: ImageData;
        blobUrl: string;
    } | null> {
        if (!videoElement || videoElement.videoWidth === 0) return null;

        // 1. Create a temporary offscreen canvas
        const canvas = document.createElement("canvas");
        canvas.width = videoElement.videoWidth;
        canvas.height = videoElement.videoHeight;

        const ctx = canvas.getContext("2d", { willReadFrequently: true });
        if (!ctx) return null;

        // 2. Draw the current video frame onto the canvas
        ctx.drawImage(videoElement, 0, 0, canvas.width, canvas.height);

        // 3. Extract the pixel data (synchronous)
        const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);

        // 4. Create the Blob URL (asynchronous)
        const blobUrl = await new Promise<string | null>((resolve) => {
            canvas.toBlob(
                (blob) => {
                    if (blob) {
                        resolve(URL.createObjectURL(blob));
                    } else {
                        resolve(null);
                    }
                },
                "image/jpeg",
                1.0,
            );
        });

        if (!blobUrl) return null;

        console.log("Done with the image creation and Blob URL generation");

        return { imageData, blobUrl };
    }

    onMount(async () => {
        jmuxer = new JMuxer({
            node: videoElement,
            mode: "video",
            flushingTime: 1,
            maxDelay: 200,
            fps: 30,
            debug: true,
        });

        const response = await fetch("http://localhost:5678/stream");
        const reader = response.body?.getReader();
        if (!reader) return;

        while (true) {
            const { done, value } = await reader.read();
            if (done) break;
            jmuxer.feed({ video: value });
        }
    });

    onDestroy(() => {
        if (jmuxer) jmuxer.destroy();
    });
</script>

<video bind:this={videoElement} autoplay muted playsinline controls> </video>

<style>
    video {
        width: 100%;
        max-width: 800px;
        background: black;
    }
</style>
