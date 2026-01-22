import * as path from 'path';
import * as fs from 'fs';
import { workspace, ExtensionContext, window } from 'vscode';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    TransportKind
} from 'vscode-languageclient/node';

let client: LanguageClient | undefined;

export async function activate(context: ExtensionContext): Promise<void> {
    const serverPath = getServerPath(context);

    if (!serverPath) {
        window.showErrorMessage(
            'Tekton LSP: Could not find tekton-lsp binary. ' +
            'Please install it or set tekton-lsp.serverPath in settings.'
        );
        return;
    }

    const serverOptions: ServerOptions = {
        run: {
            command: serverPath,
            transport: TransportKind.stdio
        },
        debug: {
            command: serverPath,
            args: ['--verbose'],
            transport: TransportKind.stdio
        }
    };

    const clientOptions: LanguageClientOptions = {
        documentSelector: [
            { scheme: 'file', language: 'yaml' },
            { scheme: 'file', language: 'tekton-yaml' },
            { scheme: 'file', pattern: '**/*.yaml' },
            { scheme: 'file', pattern: '**/*.yml' }
        ],
        synchronize: {
            fileEvents: workspace.createFileSystemWatcher('**/*.{yaml,yml}')
        },
        initializationOptions: {
            // Pass any initialization options here
        }
    };

    client = new LanguageClient(
        'tekton-lsp',
        'Tekton Language Server',
        serverOptions,
        clientOptions
    );

    // Start the client. This will also launch the server
    await client.start();

    console.log('Tekton LSP client started');
}

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}

function getServerPath(context: ExtensionContext): string | undefined {
    // Check user configuration first
    const config = workspace.getConfiguration('tekton-lsp');
    const configuredPath = config.get<string>('serverPath');

    if (configuredPath && fs.existsSync(configuredPath)) {
        return configuredPath;
    }

    // Check for bundled binary in extension
    const bundledPath = path.join(context.extensionPath, 'bin', 'tekton-lsp');
    if (fs.existsSync(bundledPath)) {
        return bundledPath;
    }

    // Check for bundled binary with .exe extension (Windows)
    const bundledPathExe = bundledPath + '.exe';
    if (fs.existsSync(bundledPathExe)) {
        return bundledPathExe;
    }

    // Try to find in PATH
    const pathEnv = process.env.PATH || '';
    const pathDirs = pathEnv.split(path.delimiter);

    for (const dir of pathDirs) {
        const candidate = path.join(dir, 'tekton-lsp');
        if (fs.existsSync(candidate)) {
            return candidate;
        }
        // Windows
        const candidateExe = candidate + '.exe';
        if (fs.existsSync(candidateExe)) {
            return candidateExe;
        }
    }

    return undefined;
}
