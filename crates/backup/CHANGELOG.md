# Changelog
All notable changes to this project will be documented in this file. See [conventional commits](https://www.conventionalcommits.org/) for commit guidelines.

- - -
## [backup-v0.1.0](https://github.com/AMTSupport/tools/compare/5678af914fbd25777e9a28dbaaf557a016530b7d..backup-v0.1.0) - 2023-10-05
#### <!-- 0 -->⛰️  Features
- Beginning of auto reboot manager - ([ac71a2c](https://github.com/AMTSupport/tools/commit/ac71a2c9639d554b5317937b106effa09d57cd26)) - DaRacci
- parallel item fetching & Some more fixes - ([213ad05](https://github.com/AMTSupport/tools/commit/213ad0570c5d4028251c0cadb2effa729ad408cd)) - DaRacci
- Better progress bars & cli interaction - ([fce705d](https://github.com/AMTSupport/tools/commit/fce705d7e36bd3d7ec279bee9758e70f2fa9619c)) - DaRacci
- Add better progress reports and parallel run exports - ([af932af](https://github.com/AMTSupport/tools/commit/af932af794dcef2736ac2ac7c9ee8a3d7450434d)) - DaRacci
- OnePassword Exporter - ([a3016f1](https://github.com/AMTSupport/tools/commit/a3016f180a4df9453cf39cf05837fe9b23b57cd9)) - DaRacci
- Add download progress to s3 - ([0303c19](https://github.com/AMTSupport/tools/commit/0303c195e1f79d376ae2776660150b0120820d01)) - DaRacci
- bitwarden exporter - ([13a98b7](https://github.com/AMTSupport/tools/commit/13a98b7524cb229f21aaa277a3041095418a6ebb)) - DaRacci
- Large push - ([3900f31](https://github.com/AMTSupport/tools/commit/3900f31c4e6a1a829990d8ebced1c8b91c0c825c)) - DaRacci
- Offline data backup cli - ([fd58bae](https://github.com/AMTSupport/tools/commit/fd58bae2be9ed212d29b36a3f0cd8cb25d19b6ad)) - DaRacci
#### <!-- 1 -->🐛 Bug Fixes
- **(backup)** compile errors - ([d60d8ac](https://github.com/AMTSupport/tools/commit/d60d8ac176d745de1e25e659870833d5f5d1e324)) - DaRacci
- **(backup)** spelling - ([f6df373](https://github.com/AMTSupport/tools/commit/f6df3734d20b9b7465ed3b0c1109dd0ed7decd8a)) - DaRacci
- **(backup)** don't drop log guard - ([ffd2901](https://github.com/AMTSupport/tools/commit/ffd290141d8609d2c0eadf8a564e7cb09e7d7f6b)) - DaRacci
- **(backup)** compile errors - ([a3766de](https://github.com/AMTSupport/tools/commit/a3766dee51fa37857e11ff1e88bcaca5ea517c5b)) - DaRacci
- **(backup)** tests - ([0047ff4](https://github.com/AMTSupport/tools/commit/0047ff44a21ae54c4ea4199f8eaf04ec25c3fc16)) - DaRacci
- compile errors - ([cd0d478](https://github.com/AMTSupport/tools/commit/cd0d4785529084fc976fe9c1d54b62ffe918128b)) - DaRacci
- bitwarden cli - ([706af61](https://github.com/AMTSupport/tools/commit/706af61a30c9f55cbc95190a659c0d13494f6e03)) - DaRacci
- save config on modify and overwrite by default - ([3b29908](https://github.com/AMTSupport/tools/commit/3b2990883b230cd1fec08212e1698461c2e3eb58)) - DaRacci
- Zip file for 1PUX export with attributes data - ([e6866a7](https://github.com/AMTSupport/tools/commit/e6866a7e430bbc2fad4e9a304e48be0cf6be956f)) - DaRacci
- path issues on windows - ([748624e](https://github.com/AMTSupport/tools/commit/748624e9d949316aad7f8e99f1fccde582317da1)) - DaRacci
- imports - ([c8e4bfe](https://github.com/AMTSupport/tools/commit/c8e4bfe0be0c10bbcc3ce449aa2e499dcc55e863)) - DaRacci
- Handle errors after later - ([03f4443](https://github.com/AMTSupport/tools/commit/03f4443dd6c917574d8b87a2ef49dd6acf1668da)) - DaRacci
- clap help and other features - ([ca9252d](https://github.com/AMTSupport/tools/commit/ca9252d3394235a3724cb387d3d898cca16bcac2)) - DaRacci
- use built in serde feature - ([2207646](https://github.com/AMTSupport/tools/commit/220764636364be339bfa9f031e22670d26467de2)) - DaRacci
#### <!-- 2 -->🚜 Refactor
- **(backup)** clippy fix - ([fc59e6b](https://github.com/AMTSupport/tools/commit/fc59e6bed69fca1810cda47c3958040ccee8982f)) - DaRacci
- Abstraction for downloading cli directly. - ([f569fcc](https://github.com/AMTSupport/tools/commit/f569fccd9acc620060c0a8678547261460662c16)) - DaRacci
- cleanup config stuff - ([eb951f6](https://github.com/AMTSupport/tools/commit/eb951f6c4be48d5f01c17707ac04642ddd7c68c1)) - DaRacci
- cleanup and get S3 working again - ([f7e80f0](https://github.com/AMTSupport/tools/commit/f7e80f05f8b5269d3590d7dee94f3d4e49c68d8d)) - DaRacci
#### <!-- 5 -->🎨 Styling
- **(backup)** formatting - ([44d30d4](https://github.com/AMTSupport/tools/commit/44d30d4b610d3495c28bdf693fe3355388213196)) - DaRacci
- clippy fix - ([daf1c02](https://github.com/AMTSupport/tools/commit/daf1c02a2657655a992c020561f7f3006c7ccda2)) - DaRacci
#### <!-- 7 -->🤖 CI Tasks
- **(flake)** Update flake for ci and workflows - ([6d5351a](https://github.com/AMTSupport/tools/commit/6d5351a5c8fd5588bd3ea866864fe6ff72bd911f)) - DaRacci
#### <!-- 8 -->🏗️ Build Tasks
- Globalise deps using workspace - ([fcd3c05](https://github.com/AMTSupport/tools/commit/fcd3c056c79fc749701dee7e94c7819a50a56cd1)) - DaRacci
#### <!-- 9 -->⚙️ Miscellaneous Tasks
- **(backup)** clippy fix - ([78f8283](https://github.com/AMTSupport/tools/commit/78f828323af00b60623c633560c8a6364c2eca44)) - DaRacci
- **(backup)** drop tui for now - ([57d6069](https://github.com/AMTSupport/tools/commit/57d60695ba4ca931707c410877ebcc2faf741e87)) - DaRacci
- **(backup)** remove moved code - ([7b0b31a](https://github.com/AMTSupport/tools/commit/7b0b31ab289bd7071dff9c45c9c3c9853d32e3dc)) - DaRacci
- **(backup)** current progress - ([712a52d](https://github.com/AMTSupport/tools/commit/712a52d5a67d5bf24ab2c09c0a39d2565794a0ad)) - DaRacci
- **(backup)** More progress on new autoprune. - ([c51326e](https://github.com/AMTSupport/tools/commit/c51326e63c40cbac5c6a238e930b588ca74a1caa)) - DaRacci
- **(backup)** Work on autoprune - ([728dbc3](https://github.com/AMTSupport/tools/commit/728dbc3d4ef22f5e99bab06fa9354a214d5e19d9)) - DaRacci
- **(backup)** fix compile - ([28c3377](https://github.com/AMTSupport/tools/commit/28c3377ac18d5c03897b967bf7ea43c9f3fbb999)) - DaRacci
- start all versions from 0.0.1 - ([47272f8](https://github.com/AMTSupport/tools/commit/47272f8fad2c414854177f81625713634fa0cb7e)) - DaRacci
- clippy fix - ([29ec950](https://github.com/AMTSupport/tools/commit/29ec950f789f7988a1e46e6030e4c5cd4b8a93df)) - DaRacci
- Cargo.toml updates - ([468e759](https://github.com/AMTSupport/tools/commit/468e759bd2169f5185a3bc7b3cf864aaf7e02c19)) - DaRacci
- more work on the backup crate - ([95428c1](https://github.com/AMTSupport/tools/commit/95428c12889f3dd13ac7cfc79af294b86c6427e1)) - DaRacci
- more work on cleaner and a new popup helper - ([ed76db3](https://github.com/AMTSupport/tools/commit/ed76db391ff4762053e3ba4ab19b2b5670acdd14)) - DaRacci
- move to lib - ([8d7dc1b](https://github.com/AMTSupport/tools/commit/8d7dc1b9bd3eb25aad2cecb951679e1b06fda16e)) - DaRacci
- License - ([b98a3b9](https://github.com/AMTSupport/tools/commit/b98a3b924d2c1aa96e63a8bac3f87d4c239d61e3)) - DaRacci
- cleanup and macros - ([b8e857e](https://github.com/AMTSupport/tools/commit/b8e857ea6895799b48a17adc54bb3ed768baf119)) - DaRacci
- alot more work and tests stuff - ([26f93d3](https://github.com/AMTSupport/tools/commit/26f93d32e3c69eead2d842642d2c4e13e3ec6327)) - DaRacci
- cleanup, tests, and some fixes - ([afb7fde](https://github.com/AMTSupport/tools/commit/afb7fde7b173b16cec7d11b8ab024c41a16e2dbc)) - DaRacci
- god damnit. even more 1password shit - ([22a6cc0](https://github.com/AMTSupport/tools/commit/22a6cc071fe0ad3419ad35bfc672bf0bfb03a46a)) - DaRacci
- swap out simplelog for tracing - ([79e9cff](https://github.com/AMTSupport/tools/commit/79e9cff06b05820669b10967d41099490d799afd)) - DaRacci
- clippy demands it - ([8f0890b](https://github.com/AMTSupport/tools/commit/8f0890bd9b0f72d583ffe77c4fbcdbd9212b19c7)) - DaRacci
- run format - ([a3e25f3](https://github.com/AMTSupport/tools/commit/a3e25f39780776deaf47726c77f2ff275c7efb42)) - DaRacci
- must. satisfy. the. clippy. - ([91ba822](https://github.com/AMTSupport/tools/commit/91ba822ce914db2635e97b41785edcb1f740f6e4)) - DaRacci
- some fixes - ([a70965b](https://github.com/AMTSupport/tools/commit/a70965bac8f965c43f38e399c02bb391c3bb4629)) - DaRacci
- Fix spelling mistakes - ([8319ca9](https://github.com/AMTSupport/tools/commit/8319ca9acd2538fa8a5769e4e0fc15844e8fa533)) - DaRacci
- Move config to subdir and break into multiple files - ([77110c9](https://github.com/AMTSupport/tools/commit/77110c9093811f617526e51d7281d690e108961a)) - DaRacci
- cleanup imports and such - ([b1a167e](https://github.com/AMTSupport/tools/commit/b1a167e9c0e530092ea43e83c8292e3f2eefdc37)) - DaRacci
- helper functions - ([055985f](https://github.com/AMTSupport/tools/commit/055985fe0c9c67293fd1efcfa3a8215a1f2cd5a0)) - DaRacci

- - -

Changelog generated by [cocogitto](https://github.com/cocogitto/cocogitto).