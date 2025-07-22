# プロジェクト配布ガイド

> Librarianアプリケーションの配布用ビルドとインストーラー作成のための包括的なガイド

## 概要

このガイドは、Librarian（Tauri v2）アプリケーションを各プラットフォーム（Windows、macOS、Linux）で配布可能な形式にビルドし、インストーラーを作成する方法を説明します。

## 配布前のチェックリスト

### 1. プロジェクトの準備

```bash
# 依存関係の確認
npm audit
cargo audit

# 全テストの実行
npm run test:run
cargo test --all

# コード品質チェック
npm run quality
cargo clippy

# ビルドテスト
npm run build
cargo build --release
```

### 2. メタデータの確認

**tauri.conf.json**の設定確認：

```json
{
  "productName": "Librarian",
  "version": "1.0.0",
  "identifier": "com.librarian.app",
  "bundle": {
    "active": true,
    "category": "DeveloperTool",
    "copyright": "© 2025 Librarian Development Team",
    "shortDescription": "AI Agent Prompt Management Tool",
    "longDescription": "Desktop application for managing AI agent prompts with MCP protocol support"
  }
}
```

## プラットフォーム別ビルド

### macOS

#### 1. 基本ビルド

```bash
# 開発者モードでのビルド
npm run tauri build

# リリースモード（コード署名なし）
npm run tauri build -- --target universal-apple-darwin
```

#### 2. コード署名（推奨）

```bash
# Apple Developer Certificateが必要
export APPLE_CERTIFICATE_PASSWORD="your_password"
export APPLE_ID="your_apple_id@email.com"
export APPLE_PASSWORD="app_specific_password"

# 署名付きビルド
npm run tauri build -- --target universal-apple-darwin
```

#### 3. 公証（Notarization）

```bash
# アプリ公証
xcrun notarytool submit \
  "src-tauri/target/universal-apple-darwin/release/bundle/dmg/Librarian_1.0.0_universal.dmg" \
  --apple-id "your_apple_id@email.com" \
  --password "app_specific_password" \
  --team-id "your_team_id" \
  --wait

# 公証結果の確認
xcrun stapler staple "src-tauri/target/universal-apple-darwin/release/bundle/dmg/Librarian_1.0.0_universal.dmg"
```

#### 4. macOS配布物

生成されるファイル：

- `Librarian_1.0.0_universal.dmg` - DMGインストーラー
- `Librarian.app` - アプリケーションバンドル

### Windows

#### 1. 基本ビルド

```bash
# Windows MSIインストーラー
npm run tauri build -- --target x86_64-pc-windows-msvc

# Windows EXEインストーラー
npm run tauri build -- --bundles nsis
```

#### 2. コード署名

```powershell
# Windows SDK Sign Tool使用
$cert_path = "path/to/certificate.p12"
$password = "certificate_password"

# 署名付きビルド
$env:WINDOWS_CERTIFICATE_PATH = $cert_path
$env:WINDOWS_CERTIFICATE_PASSWORD = $password

npm run tauri build
```

#### 3. Windows配布物

生成されるファイル：

- `Librarian_1.0.0_x64.msi` - MSIインストーラー
- `Librarian_1.0.0_x64-setup.exe` - NSIS EXEインストーラー

### Linux

#### 1. 基本ビルド

```bash
# AppImageビルド
npm run tauri build -- --target x86_64-unknown-linux-gnu

# Debian/Ubuntuパッケージ
npm run tauri build -- --bundles deb

# Red Hat/Fedoraパッケージ
npm run tauri build -- --bundles rpm
```

#### 2. Linux配布物

生成されるファイル：

- `librarian_1.0.0_amd64.AppImage` - AppImage（ポータブル）
- `librarian_1.0.0_amd64.deb` - Debianパッケージ
- `librarian-1.0.0-1.x86_64.rpm` - RPMパッケージ

## カスタムインストーラー

### NSISスクリプト（Windows）

`src-tauri/nsis/installer.nsi`を作成：

```nsis
!define PRODUCT_NAME "Librarian"
!define PRODUCT_VERSION "1.0.0"
!define PRODUCT_PUBLISHER "Librarian Development Team"
!define PRODUCT_WEB_SITE "https://github.com/your-repo/librarian"

; インストーラー設定
Name "${PRODUCT_NAME} ${PRODUCT_VERSION}"
OutFile "LibrarianInstaller-${PRODUCT_VERSION}.exe"
InstallDir "$PROGRAMFILES64\\${PRODUCT_NAME}"

; セクション
Section "MainSection" SEC01
  SetOutPath "$INSTDIR"
  File /r "target\\release\\*"

  ; ショートカット作成
  CreateDirectory "$SMPROGRAMS\\${PRODUCT_NAME}"
  CreateShortCut "$SMPROGRAMS\\${PRODUCT_NAME}\\${PRODUCT_NAME}.lnk" "$INSTDIR\\librarian.exe"
  CreateShortCut "$DESKTOP\\${PRODUCT_NAME}.lnk" "$INSTDIR\\librarian.exe"
SectionEnd
```

### macOS DMG カスタマイゼーション

`src-tauri/dmg/dmg-spec.json`を作成：

```json
{
  "title": "Librarian Installer",
  "icon": "icons/icon.icns",
  "background": "dmg/background.png",
  "window": {
    "size": {
      "width": 540,
      "height": 380
    }
  },
  "contents": [
    {
      "x": 140,
      "y": 120,
      "type": "file",
      "path": "Librarian.app"
    },
    {
      "x": 400,
      "y": 120,
      "type": "link",
      "path": "/Applications"
    }
  ]
}
```

## 自動ビルドパイプライン

### GitHub Actions

`.github/workflows/build-release.yml`：

```yaml
name: 'Build and Release'

on:
  push:
    tags:
      - 'v*'

jobs:
  release:
    permissions:
      contents: write
    strategy:
      fail-fast: false
      matrix:
        platform: [macos-latest, ubuntu-20.04, windows-latest]

    runs-on: ${{ matrix.platform }}
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4

      - name: Setup Node
        uses: actions/setup-node@v4
        with:
          node-version: 18

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Install dependencies (Ubuntu)
        if: matrix.platform == 'ubuntu-20.04'
        run: |
          sudo apt-get update
          sudo apt-get install -y libgtk-3-dev libwebkit2gtk-4.0-dev librsvg2-dev

      - name: Install frontend dependencies
        run: npm ci

      - name: Build the app
        uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          tagName: ${{ github.ref_name }}
          releaseName: 'Librarian ${{ github.ref_name }}'
          releaseBody: 'See the assets to download this version and install.'
          releaseDraft: true
          prerelease: false
```

## 配布テスト

### 1. インストールテスト

各プラットフォームでインストールテストを実行：

```bash
# macOS
open Librarian_1.0.0_universal.dmg

# Windows
./Librarian_1.0.0_x64-setup.exe /S

# Linux
sudo dpkg -i librarian_1.0.0_amd64.deb
```

### 2. 機能テスト

インストール後の基本機能確認：

- アプリケーション起動
- MCPサーバー機能
- ファイル読み書き権限
- ネットワーク接続

### 3. セキュリティテスト

```bash
# VirusTotalでのスキャン
# https://www.virustotal.com/

# Windowsでのスマートスクリーン確認
# 初回実行時の警告確認

# macOSでのGatekeeper確認
spctl -a -v /Applications/Librarian.app
```

## 配布チャネル

### 1. GitHub Releases

```bash
# GitHub CLIを使用したリリース作成
gh release create v1.0.0 \
  --title "Librarian v1.0.0" \
  --notes-file RELEASE_NOTES.md \
  src-tauri/target/universal-apple-darwin/release/bundle/dmg/*.dmg \
  src-tauri/target/x86_64-pc-windows-msvc/release/bundle/msi/*.msi \
  src-tauri/target/x86_64-unknown-linux-gnu/release/bundle/deb/*.deb
```

### 2. 公式パッケージストア

- **macOS**: Mac App Store（Apple Developer登録必要）
- **Windows**: Microsoft Store（Developer登録必要）
- **Linux**: Snapcraft、Flatpak、AURなど

### 3. パッケージマネージャー

```bash
# Homebrew (macOS)
# Formulaの作成とPR

# Chocolatey (Windows)
# パッケージ定義の作成

# APT (Ubuntu/Debian)
# PPAリポジトリの作成
```

## 更新配信システム

### Tauri Updater実装

`src-tauri/tauri.conf.json`：

```json
{
  "updater": {
    "active": true,
    "endpoints": ["https://releases.example.com/update/{{target}}/{{current_version}}"],
    "dialog": true,
    "pubkey": "your_public_key_here"
  }
}
```

### 更新サーバー実装例

```javascript
// Express.js更新サーバー
app.get('/update/:target/:version', (req, res) => {
  const { target, version } = req.params;
  const latestVersion = getLatestVersion();

  if (isNewerVersion(latestVersion, version)) {
    res.json({
      version: latestVersion,
      notes: getReleaseNotes(latestVersion),
      pub_date: getReleaseDate(latestVersion),
      platforms: {
        [target]: {
          signature: getSignature(target, latestVersion),
          url: getDownloadUrl(target, latestVersion),
        },
      },
    });
  } else {
    res.status(204).end();
  }
});
```

## まとめ

配布プロセスのベストプラクティス：

1. **品質保証**: 必ず全テストを通してからビルド
2. **コード署名**: セキュリティと信頼性のため推奨
3. **自動化**: CI/CDパイプラインで人的ミスを削減
4. **テスト**: 実環境での動作確認を徹底
5. **ドキュメント**: ユーザー向けインストール手順を準備

継続的な改善と監視により、安定した配布プロセスを維持しましょう。
