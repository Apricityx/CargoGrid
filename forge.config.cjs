module.exports = {
    packagerConfig: {
        icon: './src/assets/icon',
    },
    rebuildConfig: {},
    makers: [
        {
            name: '@electron-forge/maker-zip',
            platforms: ['darwin', 'win32'],
        },
    ],
};