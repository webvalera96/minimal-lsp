// The module 'vscode' contains the VS Code extensibility API
// Import the module and reference it with the alias vscode in your code below
import * as net from 'net';
import * as vscode from 'vscode';
import * as path from 'path';
import { workspace, ExtensionContext, window as Window } from 'vscode';

import {
	LanguageClient,
	LanguageClientOptions,
	ServerOptions,
	TransportKind,
	StreamInfo,
} from 'vscode-languageclient/node';

let client: LanguageClient;

// This method is called when your extension is activated
// Your extension is activated the very first time the command is executed
export function activate(context: vscode.ExtensionContext) {

	const connectionInfo = {
		host: '0.0.0.0',
		port: 9090,
	};


	const serverOptions: ServerOptions = () => {
		return new Promise<StreamInfo>((resolve,reject) => {
			const clientSocket = new net.Socket();

			clientSocket.connect(connectionInfo.port, connectionInfo.host, () => {
				resolve({
					reader: clientSocket,
					writer: clientSocket,
				});
			});

			clientSocket.on('error', (err) => {
				reject(err);
			});
		});
	};


	// Options to control the language client
	const clientOptions: LanguageClientOptions = {
		// Register the server for plain text documents
		documentSelector: [{ scheme: 'file', language: 'plaintext' }],
		synchronize: {
			// Notify the server about file changes to '.clientrc files contained in the workspace
			fileEvents: workspace.createFileSystemWatcher('**/.clientrc')
		}
	};

	// Create the language client and start the client.

	// Start the client. This will also launch the server
	try {
		client = new LanguageClient(
			'minmalLSP',
			'Minimal LSP Example',
			serverOptions,
			clientOptions,
			true,
		);
	} catch {
		Window.showErrorMessage(`The extension couldn't be started. See the output channel for details.`);
		return;
	}
	client.start();
}

// This method is called when your extension is deactivated
export function deactivate(): Thenable<void> | undefined {
	if (!client) {
		return undefined;
	}
	return client.stop();
}
