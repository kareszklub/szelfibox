<script lang="ts">
    import VideoCanvas from "$lib/components/VideoCanvas.svelte";
    import { box } from "$lib/state.svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { listen } from "@tauri-apps/api/event";
    import { onMount } from "svelte";

    import * as Card from "$lib/components/ui/card";
    import { Button } from "$lib/components/ui/button";
    import { Badge } from "$lib/components/ui/badge";
    import { Spinner } from "$lib/components/ui/spinner";

    import {
        Camera,
        Printer,
        Ban,
        Check,
        ArrowRight,
        QrCode,
        Play,
    } from "lucide-svelte";

    let countdown: number | null = $state(null);
    let videoCanvas: VideoCanvas | null = $state(null);

    onMount(async () => {
        await listen<number>("button", (event: any) => {
            if (parseInt(event.payload) == 1) {
                onLeftButtonDown();
            } else {
                onRightButtonDown();
            }
        });
    });

    async function takePicture() {
        revokeURLS();

        const responseBuffer = await invoke<ArrayBuffer>("take_picture");

        const view = new DataView(responseBuffer);

        const imgLen = view.getUint32(0, true);

        const mainStart = 4;
        const qrStart = mainStart + imgLen;

        const qrPixels = new Uint8Array(responseBuffer.slice(qrStart));
        const qrBlob = new Blob([qrPixels], { type: "image/png" });
        box.qrBlobURL = URL.createObjectURL(qrBlob);

        const imageBlob = new Blob(
            [new Uint8Array(responseBuffer.slice(mainStart, qrStart))],
            { type: "image/png" },
        );
        box.imageBlobURL = URL.createObjectURL(imageBlob);
    }

    const revokeURLS = () => {
        if (box.imageBlobURL) {
            URL.revokeObjectURL(box.imageBlobURL);
            box.imageBlobURL = null;
        }
        if (box.qrBlobURL) {
            URL.revokeObjectURL(box.qrBlobURL);
            box.qrBlobURL = null;
        }
    };

    const onKeyDown = (e: KeyboardEvent) => {
        if (e.key === "a") {
            onLeftButtonDown();
        } else if (e.key === "d") {
            onRightButtonDown();
        }
    };

    const onLeftButtonDown = async () => {
        if (countdown) return;
        console.log("[Left] button pressed");
        if (box.stage === 1) {
            box.stage = 2;
        } else if (box.stage === 2) {
            box.freeze = false;
            countdown = 3;
            let clear = setInterval(async () => {
                if (!countdown) return;
                countdown--;
                if (countdown === 0) {
                    clearInterval(clear);
                    countdown = null;

                    console.log("Csinálom a képet, csíz!");

                    box.freeze = true;
                    await takePicture();
                }
            }, 1000);
        } else if (box.stage === 3) {
            box.stage = 4;
        } else if (box.stage === 4) {
            console.log("Indítom a nyomtatást");
            await invoke("print_picture");

            revokeURLS();
            box.freeze = false;

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
            if (!box.imageBlobURL) {
                console.error("Először csinálj egy képet!");
                return;
            }
            box.stage = 3;
        } else if (box.stage === 3) {
            box.stage = 4;
        } else if (box.stage === 4) {
            revokeURLS();
            box.freeze = false;

            box.stage = 1;
        } else {
            throw new Error("Unreachable");
        }
    };
</script>

<svelte:window onkeydown={onKeyDown} />

<div class="w-full max-w-5xl mx-auto">
    {#if box.stage === 1}
        <Card.Root class="text-center shadow-lg border-2">
            <Card.Header class="py-16">
                <Card.Title class="text-6xl font-extrabold text-primary mb-4"
                    >Szia!</Card.Title
                >
                <Card.Description class="text-2xl">
                    A kezdéshez nyomd meg az egyik gombot.
                </Card.Description>
            </Card.Header>
            <Card.Content class="flex justify-center pb-12">
                <Button
                    size="lg"
                    class="text-xl px-8 py-8 animate-pulse"
                    onclick={() => (box.stage = 2)}
                >
                    <Play class="mr-2 h-6 w-6" /> Indítás
                </Button>
            </Card.Content>
        </Card.Root>
    {:else}
        <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
            <div class="lg:col-span-2">
                <Card.Root
                    class="overflow-hidden shadow-md h-full flex flex-col justify-center bg-black/5 relative"
                >
                    {#if countdown}
                        <div
                            class="absolute inset-0 z-50 flex items-center justify-center"
                        >
                            <p
                                class="text-[10rem] font-bold text-white drop-shadow-2xl animate-bounce"
                            >
                                {countdown}
                            </p>
                        </div>
                    {/if}

                    <div
                        class="p-2 flex justify-center items-center h-full min-h-[480px]"
                    >
                        {#if box.stage === 3}
                            <div
                                class="text-center bg-white p-8 rounded-xl shadow-sm"
                            >
                                {#if box.qrBlobURL}
                                    <img
                                        src={box.qrBlobURL}
                                        alt="QR Code"
                                        class="w-64 h-64 object-contain mx-auto"
                                    />
                                {:else}
                                    <div
                                        class="w-64 h-64 flex items-center justify-center"
                                    >
                                        <QrCode
                                            class="h-16 w-16 text-muted-foreground animate-spin"
                                        />
                                    </div>
                                {/if}
                                <p class="mt-4 font-semibold text-lg">
                                    Olvasd be a képet!
                                </p>
                            </div>
                        {:else}
                            <div
                                class="relative rounded-lg overflow-hidden border-4 border-muted"
                            >
                                {#if box.freeze}
                                    {#if box.imageBlobURL}
                                        <img
                                            src={box.imageBlobURL}
                                            alt="Taken pic"
                                        />
                                    {:else}
                                        <Spinner class="size-16" />
                                    {/if}
                                {:else}
                                    <VideoCanvas
                                        bind:this={videoCanvas}
                                        width={720}
                                        height={480}
                                    />
                                {/if}
                            </div>
                        {/if}
                    </div>
                </Card.Root>
            </div>

            <div class="flex flex-col gap-4">
                <Card.Root>
                    <Card.Header>
                        <Card.Title class="flex justify-between items-center">
                            Instrukciók
                            <Badge variant="outline" class="text-lg px-3">
                                Lépés {box.stage} / 4
                            </Badge>
                        </Card.Title>
                    </Card.Header>
                    <Card.Content>
                        <p class="text-xl font-medium leading-relaxed">
                            {#if box.stage === 2}
                                Készíts egy képet, és ha tetszik, menj tovább!
                            {:else if box.stage === 3}
                                Szkenneld be a QR kódot a telefonoddal a kép
                                mentéséhez.
                            {:else if box.stage === 4}
                                Szeretnéd kinyomtatni a fényképet emlékbe?
                            {/if}
                        </p>
                    </Card.Content>
                </Card.Root>

                <Card.Root
                    class="flex-grow flex flex-col justify-center bg-muted/30"
                >
                    <Card.Header>
                        <Card.Title>Gombok</Card.Title>
                    </Card.Header>
                    <Card.Content class="grid gap-4">
                        <div
                            class="flex items-center gap-4 p-4 border rounded-lg bg-background shadow-sm border-l-8 border-l-red-500"
                        >
                            <div
                                class="h-12 w-12 rounded-full bg-red-500 flex items-center justify-center text-white font-bold text-xl shadow-inner"
                            >
                                L
                            </div>
                            <div class="flex-1">
                                <p
                                    class="text-sm text-muted-foreground uppercase font-bold tracking-wider"
                                >
                                    Bal Gomb
                                </p>
                                <p
                                    class="text-xl font-bold flex items-center gap-2"
                                >
                                    {#if box.stage === 2}
                                        <Camera class="h-5 w-5" /> Új kép
                                    {:else if box.stage === 3}
                                        <ArrowRight class="h-5 w-5" /> Tovább
                                    {:else if box.stage === 4}
                                        <Printer class="h-5 w-5" /> Nyomtatás
                                    {/if}
                                </p>
                            </div>
                        </div>

                        <div
                            class="flex items-center gap-4 p-4 border rounded-lg bg-background shadow-sm border-l-8 border-l-blue-500"
                        >
                            <div
                                class="h-12 w-12 rounded-full bg-blue-500 flex items-center justify-center text-white font-bold text-xl shadow-inner"
                            >
                                R
                            </div>
                            <div class="flex-1">
                                <p
                                    class="text-sm text-muted-foreground uppercase font-bold tracking-wider"
                                >
                                    Jobb Gomb
                                </p>
                                <p
                                    class="text-xl font-bold flex items-center gap-2"
                                >
                                    {#if box.stage === 2}
                                        <Check class="h-5 w-5" /> Kész
                                    {:else if box.stage === 3}
                                        <ArrowRight class="h-5 w-5" /> Tovább
                                    {:else if box.stage === 4}
                                        <Ban class="h-5 w-5" /> Nem kérem
                                    {/if}
                                </p>
                            </div>
                        </div>
                    </Card.Content>
                </Card.Root>
            </div>
        </div>
    {/if}
</div>
