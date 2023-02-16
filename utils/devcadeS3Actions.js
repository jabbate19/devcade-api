const fs = require('fs');
const path = require('path');
const config = require('./config');
const Minio = require('minio');
const mime = require('mime-types');

// const DOWNLOADS_DIR = path.join(__dirname, '/../downloads');
// const UPLOADS_DIR = path.join(__dirname, '/../uploads');

const DOWNLOADS_DIR = '/tmp/downloads';
const UPLOADS_DIR = '/tmp/uploads';

const minioClientConfig = {
    endPoint: config.S3_ENDPOINT,
    useSSL: true,
    accessKey: config.S3_ACCESSKEYID,
    secretKey: config.S3_SECRETACCESSKEY
};


const uploadGameFile = async (gameId, basename, isMediaFile = false) => {
    try {
        const s3 = new Minio.Client(minioClientConfig);

        const file_path = `${UPLOADS_DIR}/${gameId}`;//file_name.replace('\\', '/').split('/').slice(0, -1).join('/');
        const file_name = `${file_path}/${basename}`;

        if (!fs.existsSync(file_path) || !fs.existsSync(file_name)) {
            throw "ERRMISSING";
        }

        var fileStream = fs.createReadStream(file_name);
        fileStream.on('error', (err) => {
            console.log('File Error', err)
        });

        var fileStat = await fs.stat(file_name, async (err, stats) => {
            if (err) {
                throw err;
            }
            if (isMediaFile) {
                await s3.putObject(
                    config.S3_GAMES_BUCKET, 
                    `${gameId}/${basename}`,
                    fileStream,
                    stats.size,
                    { "Content-Type": mime.lookup(basename) },
                    (err, objInfo) => {
                        if (err) {
                            throw err;
                        }
                        console.log('Upload Success', objInfo);
                    });
            } else {
                await s3.putObject(
                    config.S3_GAMES_BUCKET, 
                    `${gameId}/${basename}`,
                    fileStream,
                    stats.size,
                    (err, objInfo) => {
                        if (err) {
                            throw err;
                        }
                        console.log('Upload Success', objInfo);
                    });
            }
            
        });
    } catch (e) {
        console.log('Error:', e);
        throw e;
    }
};

const downloadGame = async (gameId) => {
    try {
        const s3 = new Minio.Client(minioClientConfig);

        // create download directory if it does not exist
        const file_path = `${DOWNLOADS_DIR}/${gameId}`
        console.log(file_path);
        if (!fs.existsSync(file_path)) {
            fs.mkdirSync(file_path, { recursive: true });
        }

        // Download the game zip
        const file_dest = `${file_path}/${gameId}.zip`;
        const key = `${gameId}/${gameId}.zip`;
        console.log("dest")
        console.log(file_dest);
        console.log('key')
        console.log(key)
            await s3.fGetObject(
                config.S3_GAMES_BUCKET, 
                key,
                file_dest,
                (err) => {
                    if (err) {
                        throw err
                    }
                    console.log(`Downloaded ${key}`);
                });
        return true;
    } catch (e) {
        console.log('Error:', e);
        return false;
    }
};


const getIconS3Link = async (gameId) => {
    return await getMediaS3Link(gameId, 'icon');
};

const getBannerS3Link = async (gameId) => {
    return await getMediaS3Link(gameId, 'banner');
};

const getMediaS3Link = async (gameId, mediaType) => {
    const media = (await getGamesBucketObjects(gameId)).find(key => key.name.includes(mediaType));
    if (media) {
        const mediaBasename = path.basename(media.name);
        return `https://${config.S3_ENDPOINT}/${config.S3_GAMES_BUCKET}/${gameId}/${mediaBasename}`;
    }
    return "";
}

/**
 * Checks if game exists in the s3 bucket
 * @param {*} gameId id of game bucket link being fetched
 * @returns link for game's s3 bucket
 */
const getGameZipS3Link = async (gameId) => {
    const s3 = new Minio.Client(minioClientConfig);
    // check if bucket for gameId exists
    if (await s3.bucketExists(`${config.S3_GAMES_BUCKET}/${gameId}/${gameId}.zip`)) {
        return `https://${config.S3_ENDPOINT}/${config.S3_GAMES_BUCKET}/${gameId}/${gameId}.zip`;
    }

};

const downloadMedia = async (gameId, mediaType) => {
    try {
        const s3 = new Minio.Client(minioClientConfig);
        // create download directory if it does not exist
        const file_path = `${DOWNLOADS_DIR}/${gameId}/`
        if (!fs.existsSync(file_path)) {
            fs.mkdirSync(file_path, { recursive: true });
        }

        const mediaBasename = path.basename((await getGamesBucketObjects(gameId))
            .find(key => key.name.includes(mediaType)).name);

        // Download the game zip
        const file_dest = `${file_path}/medias/${mediaBasename}`;
        console.log(file_dest);
        const key = `${gameId}/${mediaBasename}`;
            await s3.fGetObject(
                config.S3_GAMES_BUCKET, 
                key,
                file_dest,
                (err) => {
                    if (err) {
                        throw err
                    }
                    console.log(`Downloaded ${key}`);
                });
        return true;
    } catch (e) {
        console.log('Error:', e);
        return false;
    }
};

const downloadBanner = async (gameId) => {
    return await downloadMedia(gameId, 'banner');
};

const downloadIcon = async (gameId) => {
    return await downloadMedia(gameId, 'icon');
};

const waitForFile = async (gameId, localFileDir, gameFileType) => {
    const basename = path.basename((await getGamesBucketObjects(gameId))
        .find(key => key.name.includes(gameFileType)).name);

    console.log(`${localFileDir}/${basename}`);
    while (!fs.existsSync(`${localFileDir}/${basename}`)) {
        console.log('waiting');
        await delay(250);
    }
}

const waitForGame = async (gameId) => {
    while (!fs.existsSync(`${DOWNLOADS_DIR}/${gameId}/${gameId}.zip`)) {
        await delay(250);
        console.log("still waiting");
    }
};

const waitForIcon = async (gameId) => {
    await waitForFile(gameId, `${DOWNLOADS_DIR}/${gameId}/medias`, 'icon');
};

const waitForBanner = async (gameId) => {
    await waitForFile(gameId, `${DOWNLOADS_DIR}/${gameId}/medias`, 'banner');
};

const getBannerLocalPath = async (gameId) => {
    const basename = path.basename((await getGamesBucketObjects(gameId))
        .find(key => key.name.includes('banner')).name);
    return `${DOWNLOADS_DIR}/${gameId}/medias/${basename}`;
}

const getIconLocalPath = async (gameId) => {
    const basename = path.basename((await getGamesBucketObjects(gameId))
        .find(key => key.name.includes('icon')).name);
    return `${DOWNLOADS_DIR}/${gameId}/medias/${basename}`;
}

/**
 * Pauses current thread for time ms
 * @param {int} time in ms 
 * @returns 
 */
 const delay = (time) => {
    return new Promise(resolve => {
        setTimeout(resolve, time);
    });
}

const downloadMedias = async (gameId) => {
    try {
        const s3 = new Minio.Client(minioClientConfig);
        // create download directory if it does not exist
        const file_path = `${DOWNLOADS_DIR}/${gameId}/medias`
        if (!fs.existsSync(file_path)) {
            fs.mkdirSync(file_path, { recursive: true });
        }

        // Get the s3 keys of the game's icon and banner
        const mediaKeys = (await getBucketObjects(config.S3_GAMES_BUCKET, gameId)).filter(obj => 
            obj.name.includes(`${gameId}/icon`) || obj.name.includes(`${gameId}/banner`)
        ).map(media => media.name);

        // Download each object
        await mediaKeys.forEach(async (key, i) => {
            const file_dest = `${file_path}/${path.basename(key)}`;
            await s3.fGetObject(
                config.S3_GAMES_BUCKET, 
                key,
                file_dest,
                (err) => {
                    if (err) {
                        throw err
                    }
                    console.log(`Downloaded ${key}`);
                });
        });
        return true;
    } catch (e) {
        console.log('Error:', e);
        return false;
    }
};

const getAllBuckets = () => {
    try {
        const s3 = new Minio.Client(minioClientConfig);
        s3.listBuckets((err, buckets) => {
            console.log(buckets);
        });
    } catch (e) {
        console.log('Error:', e);
    }
};

const getBucketObjects = async (bucket, prefix = '') => {
    const s3 = new Minio.Client(minioClientConfig);
    const objectsList = await new Promise((resolve, reject) => {
        const objectsListTemp = [];
        
        var stream = s3.listObjectsV2(bucket, prefix, true, '')
        
        stream.on('data', (obj) => objectsListTemp.push(obj));
        stream.on('error', (err) => reject(err));
        stream.on('end', () => resolve(objectsListTemp));
    });
    //console.log(objectsList);
    return objectsList;
};

const getGamesBucketObjects = async (gameId) => {
    return await getBucketObjects(config.S3_GAMES_BUCKET, gameId);
}

const deleteGameFiles = async (gameId) => {
    const s3 = new Minio.Client(minioClientConfig);
    var objectsList = [];
    var objectsStream = s3.listObjectsV2(config.S3_GAMES_BUCKET, gameId, true, '');

    objectsStream.on('data', (obj) => {
        objectsList.push(obj.name);
    });

    objectsStream.on('error', (err) => {
        console.log(err);
    });

    objectsStream.on('end', () => {
        s3.removeObjects(config.S3_GAMES_BUCKET, objectsList, (err) => {
            if (err) {
                return console.log(`Unable to delete game files for game with ${gameId}`);
            }
            console.log(`Successfully deleted game with id ${gameId}`);
        });
    });
};

module.exports = {
    uploadGameFile,
    downloadGame,
    downloadMedias,
    downloadBanner, 
    downloadIcon,
    getGameZipS3Link,
    getGamesBucketObjects,
    getAllBuckets,
    waitForGame,
    waitForIcon,
    waitForBanner,
    delay,
    getBannerLocalPath,
    getIconLocalPath,
    getIconS3Link,
    getBannerS3Link,
    deleteGameFiles,
    UPLOADS_DIR,
    DOWNLOADS_DIR
};