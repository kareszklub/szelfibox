export let box: BoxState = $state({ stage: 1, freeze: false, qrBlobURL: null, imageBlobURL: null });

type BoxState = {
    stage: number,
    freeze: boolean,
    qrBlobURL: string | null,
    imageBlobURL: string | null,
};
