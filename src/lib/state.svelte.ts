export let box: BoxState = $state({ stage: 1, freeze: false, imageData: null });

type BoxState = {
    stage: number,
    freeze: boolean,
    imageData: ImageData | null,
};
