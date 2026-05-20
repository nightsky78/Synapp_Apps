import { expect, test } from '@playwright/test';
import { readFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

type PluginManifest = {
  executionInterface: {
    wasmTarget: string;
    wasmPath: string;
    exports: Array<{ name: string }>;
  };
  semanticInterface: {
    tools: Array<{ name: string; capability: string }>;
  };
  definitions: Record<string, unknown>;
};

type AppManifest = {
  app_id: string;
  runtime: string;
  platform_api: string;
  entrypoint: string;
  tools: Array<{ name: string }>;
  ui?: {
    entrypoint?: string;
    contributions?: Array<{ type?: string }>;
  };
  ui_schemas?: {
    main?: {
      schema_type?: string;
      resource?: {
        uri?: string;
        mime_type?: string;
      };
      page_layout_schema?: {
        layout?: string;
        components?: Array<{
          kind?: string;
          id?: string;
          affordances?: {
            draft_lifecycle?: {
              create_tool?: string;
              update_tool?: string;
              send_tool?: string;
            };
            reply_and_reply_all?: {
              tool?: string;
              reply_all_field?: string;
            };
            search_discovery?: {
              tool?: string;
            };
          };
        }>;
      };
    };
  };
};

type CatalogEntry = {
  app_id: string;
  manifest: AppManifest;
};

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const mailRoot = path.join(repoRoot, 'apps', 'first-party', 'mail-client');

test.describe('Mail Client static app contract', () => {
  test('TC-SA-MAIL-CONTRACT-001: execution endpoints stay declared across plugin semantic and package manifest', async () => {
    const plugin = await readJson<PluginManifest>(path.join(mailRoot, 'plugin.json'));
    const appManifest = await readJson<AppManifest>(path.join(mailRoot, 'synapp.app.json'));
    const catalogEntry = await readJson<CatalogEntry>(path.join(repoRoot, 'catalog', 'apps', 'mail-client.v1.json'));

    const pluginExports = sorted(plugin.executionInterface.exports.map((item) => item.name));
    const semanticTools = sorted(plugin.semanticInterface.tools.map((item) => item.name));
    const appTools = sorted(appManifest.tools.map((item) => item.name));
    const catalogTools = sorted(catalogEntry.manifest.tools.map((item) => item.name));

    expect(semanticTools).toEqual(pluginExports);
    expect(appTools).toEqual(pluginExports);
    expect(catalogTools).toEqual(pluginExports);
  });

  test('TC-SA-MAIL-CONTRACT-002: ui schema envelope exposes draft lifecycle, reply-all, and search affordances', async () => {
    const appManifest = await readJson<AppManifest>(path.join(mailRoot, 'synapp.app.json'));
    const catalogEntry = await readJson<CatalogEntry>(path.join(repoRoot, 'catalog', 'apps', 'mail-client.v1.json'));

    for (const manifest of [appManifest, catalogEntry.manifest]) {
      expect(manifest.runtime).toBe('wasm32-wasip1');
      expect(manifest.platform_api).toBe('v1');
      expect(manifest.ui?.entrypoint).toBe('ui/dist/index.html');
      expect(manifest.ui?.contributions?.[0]?.type).toBe('navigation_item');

      const mainSchema = manifest.ui_schemas?.main;
      expect(mainSchema?.schema_type).toBe('page_layout_schema');
      expect(mainSchema?.resource?.uri).toBe('app://mail-client/ui/main');
      expect(mainSchema?.resource?.mime_type).toBe('application/vnd.synapp.page-layout+json');
      expect(mainSchema?.page_layout_schema?.layout).toBe('workspace');

      const workspace = mainSchema?.page_layout_schema?.components?.[0];
      expect(workspace?.kind).toBe('mail.workspace.v1');
      expect(workspace?.id).toBe('mail.workspace');

      const affordances = workspace?.affordances;
      expect(affordances?.draft_lifecycle?.create_tool).toBe('draft_email');
      expect(affordances?.draft_lifecycle?.update_tool).toBe('update_draft');
      expect(affordances?.draft_lifecycle?.send_tool).toBe('send_email');
      expect(affordances?.reply_and_reply_all?.tool).toBe('reply_email');
      expect(affordances?.reply_and_reply_all?.reply_all_field).toBe('reply_all');
      expect(affordances?.search_discovery?.tool).toBe('search_emails');
    }
  });

  test('TC-SA-MAIL-CONTRACT-003: bridge normalizes both legacy and structured recipient fields', async () => {
    const bridgeSource = await readFile(path.join(mailRoot, 'ui', 'src', 'bridge.js'), 'utf8');

    expect(bridgeSource).toContain('const rawFrom = typeof message.from_addr === \'string\' ? message.from_addr : message.from;');
    expect(bridgeSource).toContain('const rawTo = message.to_addrs ?? message.to ?? [];');
    expect(bridgeSource).toContain('const normalizedTo = normalizeRecipientList(rawTo);');
    expect(bridgeSource).toContain('from: typeof rawFrom === \'string\'');
    expect(bridgeSource).toContain('to: normalizedTo.map((address) => {');
  });

  test('TC-SA-MAIL-CONTRACT-004: bridge send payload serializes recipients as CSV strings for platform API', async () => {
    const bridgeSource = await readFile(path.join(mailRoot, 'ui', 'src', 'bridge.js'), 'utf8');

    expect(bridgeSource).toContain('const toRecipients = serializeRecipients(draft.to, \'to\');');
    expect(bridgeSource).toContain('const ccRecipients = serializeRecipients(draft.cc, \'cc\');');
    expect(bridgeSource).toContain('const bccRecipients = serializeRecipients(draft.bcc, \'bcc\');');
    expect(bridgeSource).toContain('to: toRecipients,');
    expect(bridgeSource).toContain('cc: ccRecipients,');
    expect(bridgeSource).toContain('bcc: bccRecipients,');
  });

  test('TC-SA-MAIL-CONTRACT-005: runtime read_emails normalizes legacy and structured recipient formats', async () => {
    const calls: Array<{ url: string; body?: string }> = [];
    const bridge = await loadBridgeModule(async (input, init) => {
      const url = String(input);
      const body = typeof init?.body === 'string' ? init.body : undefined;
      calls.push({ url, body });
      if (url.startsWith('/api/v1/mail/messages?')) {
        return jsonResponse({
          messages: [
            {
              id: 'm-legacy',
              account_id: 'acc-1',
              mailbox: 'INBOX',
              from_addr: 'legacy.sender@example.com',
              to_addrs: ['legacy.to@example.com'],
              subject: 'Legacy envelope',
              date_iso: '2026-05-20T10:00:00Z',
              body_text: 'legacy body',
            },
            {
              id: 'm-structured',
              account_id: 'acc-1',
              mailbox: 'INBOX',
              from: { name: 'Structured Sender', email: 'structured.sender@example.com' },
              to: [{ name: 'Structured To', email: 'structured.to@example.com' }],
              subject: 'Structured envelope',
              date_iso: '2026-05-20T11:00:00Z',
              body_text: 'structured body',
            },
            {
              id: 'm-string-to',
              account_id: 'acc-1',
              mailbox: 'INBOX',
              from_addr: 'string.to.sender@example.com',
              to_addrs: 'string.to.one@example.com, string.to.two@example.com',
              subject: 'String recipients',
              date_iso: '2026-05-20T12:00:00Z',
              body_text: 'string to body',
            },
          ],
        });
      }
      throw new Error(`Unexpected fetch URL: ${url}`);
    });

    const result = await bridge.invoke('read_emails', {
      account_id: 'acc-1',
      mailbox_id: 'inbox',
      pagination: { limit: 50 },
    });

    const messages = result.data.snapshot.messages;
    const legacy = messages.find((message: any) => message.email_id === 'm-legacy');
    const structured = messages.find((message: any) => message.email_id === 'm-structured');
    const stringTo = messages.find((message: any) => message.email_id === 'm-string-to');

    expect(legacy?.from?.email).toBe('legacy.sender@example.com');
    expect(legacy?.to?.[0]?.email).toBe('legacy.to@example.com');
    expect(structured?.from?.email).toBe('structured.sender@example.com');
    expect(structured?.to?.[0]?.email).toBe('structured.to@example.com');
    expect(stringTo?.to?.[0]?.email).toBe('string.to.one@example.com');
    expect(stringTo?.to?.[1]?.email).toBe('string.to.two@example.com');
    expect(calls.some((call) => call.url.startsWith('/api/v1/mail/messages?'))).toBeTruthy();
  });

  test('TC-SA-MAIL-CONTRACT-006: runtime send_email posts CSV recipient lists to host API', async () => {
    const calls: Array<{ url: string; body?: string }> = [];
    const bridge = await loadBridgeModule(async (input, init) => {
      const url = String(input);
      const body = typeof init?.body === 'string' ? init.body : undefined;
      calls.push({ url, body });

      if (url === '/api/v1/mail/send') {
        return jsonResponse({ ok: true, message_id: 'sent-123' });
      }
      if (url === '/api/v1/mail/accounts') {
        return jsonResponse({
          accounts: [
            {
              account_id: 'acc-1',
              display_name: 'Synapp System Test',
              email_address: 'synapp_system_test@hettig.online',
              status: 'active',
            },
          ],
        });
      }

      throw new Error(`Unexpected fetch URL: ${url}`);
    });

    await bridge.invoke('send_email', {
      account_id: 'acc-1',
      draft: {
        account_id: 'acc-1',
        to: [{ email: 'first@example.com' }, { email: 'second@example.com' }],
        cc: [{ email: 'cc@example.com' }],
        bcc: [{ email: 'bcc@example.com' }],
        subject: 'Contract payload',
        body_text: 'Body',
      },
    });

    const sendCall = calls.find((call) => call.url === '/api/v1/mail/send');
    expect(sendCall?.body).toBeTruthy();

    const parsed = JSON.parse(sendCall!.body!);
    expect(parsed.to).toBe('first@example.com,second@example.com');
    expect(parsed.cc).toBe('cc@example.com');
    expect(parsed.bcc).toBe('bcc@example.com');
  });

  test('TC-SA-MAIL-CONTRACT-007: runtime send_email rejects malformed recipient input', async () => {
    const bridge = await loadBridgeModule(async (input) => {
      throw new Error(`Unexpected fetch URL: ${String(input)}`);
    });

    await expect(async () => {
      await bridge.invoke('send_email', {
        account_id: 'acc-1',
        draft: {
          account_id: 'acc-1',
          to: ['good@example.com\r\nBcc: injected@example.com'],
          cc: [],
          bcc: [],
          subject: 'Invalid recipient',
          body_text: 'Body',
        },
      });
    }).rejects.toThrow('to contains an invalid recipient address.');
  });
});

async function readJson<T>(filePath: string): Promise<T> {
  return JSON.parse(await readFile(filePath, 'utf8')) as T;
}

function sorted(values: string[]): string[] {
  return [...values].sort((left, right) => left.localeCompare(right));
}

async function loadBridgeModule(fetchImpl: (input: RequestInfo | URL, init?: RequestInit) => Promise<Response>) {
  (globalThis as any).window = {
    __synapp: undefined,
    localStorage: {
      _store: new Map<string, string>(),
      getItem(key: string) {
        return this._store.get(key) ?? null;
      },
      setItem(key: string, value: string) {
        this._store.set(key, value);
      },
      removeItem(key: string) {
        this._store.delete(key);
      },
    },
  };
  (globalThis as any).document = {
    querySelector() {
      return null;
    },
  };
  (globalThis as any).fetch = fetchImpl;

  const bridgePath = path.join(mailRoot, 'ui', 'src', 'bridge.js');
  const bridgeUrl = `${pathToFileURL(bridgePath).href}?t=${Date.now()}-${Math.random()}`;
  return import(bridgeUrl);
}

function jsonResponse(body: unknown): Response {
  return new Response(JSON.stringify(body), {
    status: 200,
    headers: { 'content-type': 'application/json' },
  });
}
