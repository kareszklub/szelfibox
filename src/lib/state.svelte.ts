export let box: BoxState = $state({ stage: 1, freeze: false, imageData: null, qrBlobURL: null, imageBlobURL: null });

type BoxState = {
    stage: number,
    freeze: boolean,
    imageData: ImageData | null,
    qrBlobURL: string | null,
    imageBlobURL: string | null,
};
