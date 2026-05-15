import { expect, test } from '@playwright/test';
import { readFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

type PluginManifest = {
  executionInterface: {
    wasmTarget: string;
    wasmPath: string;
    exports: Array<{ name: string }>;
  };
  semanticInterface: {
    tools: Array<{ name: string; capability: string; risk: string }>;
  };
  hostEffects: {
    required: string[];
  };
  definitions: Record<string, unknown>;
};

type AppManifest = {
  app_id: string;
  name: string;
  runtime: string;
  platform_api: string;
  entrypoint: string;
  capabilities: Array<{ capability: string; required: boolean; risk: string }>;
  tools: Array<{ name: string }>;
  ui: {
    entrypoint: string;
    contributions: Array<{
      target: string;
      page_layout_schema?: {
        components?: Array<{
          type?: string;
          required_capabilities?: string[];
          optional_capabilities?: string[];
        }>;
      };
    }>;
  };
  resource_limits: {
    memory_bytes: number;
    timeout_ms: number;
  };
};

type CatalogEntry = {
  app_id: string;
  name: string;
  version: string;
  repository_url: string;
  package_url: string;
  package_sha256: string;
  manifest_sha256: string;
  signature: { verification_status: string };
  source: { repository: string; directory: string; commit: string };
  manifest: AppManifest;
};

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const calendarRoot = path.join(repoRoot, 'apps', 'first-party', 'calendar-management');

test.describe('Calendar Management static app contract', () => {
  test('TC-SA-CAL-CONTRACT-001: plugin, app manifest, catalog, and UI bridge expose the same operation names', async () => {
    const plugin = await readJson<PluginManifest>(path.join(calendarRoot, 'plugin.json'));
    const appManifest = await readJson<AppManifest>(path.join(calendarRoot, 'synapp.app.json'));
    const catalogEntry = await readJson<CatalogEntry>(path.join(repoRoot, 'catalog', 'apps', 'calendar-management.v1.json'));
    const bridgeSource = await readFile(path.join(calendarRoot, 'ui', 'src', 'bridge.js'), 'utf8');
    const rustSource = await readFile(path.join(calendarRoot, 'src', 'lib.rs'), 'utf8');

    const pluginExports = sorted(plugin.executionInterface.exports.map((item) => item.name));
    const semanticTools = sorted(plugin.semanticInterface.tools.map((item) => item.name));
    const appTools = sorted(appManifest.tools.map((item) => item.name));
    const catalogTools = sorted(catalogEntry.manifest.tools.map((item) => item.name));
    const bridgeOperations = sorted(Array.from(bridgeSource.matchAll(/^  ([a-z][a-z0-9_]*): \(/gm), (match) => match[1]));
    const rustExports = sorted(Array.from(rustSource.matchAll(/^export_operation!\(([a-z][a-z0-9_]*)\);$/gm), (match) => match[1]));

    expect(pluginExports).toHaveLength(38);
    expect(semanticTools).toEqual(pluginExports);
    expect(appTools).toEqual(pluginExports);
    expect(catalogTools).toEqual(pluginExports);
    expect(bridgeOperations).toEqual(pluginExports);
    expect(rustExports).toEqual(pluginExports);
    expect(rustSource).toContain('pub extern "C" fn free_string');
  });

  test('TC-SA-CAL-CONTRACT-002: manifest lifecycle metadata stays inside the supported platform boundary', async () => {
    const plugin = await readJson<PluginManifest>(path.join(calendarRoot, 'plugin.json'));
    const appManifest = await readJson<AppManifest>(path.join(calendarRoot, 'synapp.app.json'));
    const catalogEntry = await readJson<CatalogEntry>(path.join(repoRoot, 'catalog', 'apps', 'calendar-management.v1.json'));

    expect(appManifest.app_id).toBe('calendar-management');
    expect(catalogEntry.app_id).toBe(appManifest.app_id);
    expect(catalogEntry.manifest.app_id).toBe(appManifest.app_id);
    expect(appManifest.runtime).toBe('wasm32-wasip1');
    expect(appManifest.platform_api).toBe('v1');
    expect(plugin.executionInterface.wasmTarget).toBe(appManifest.runtime);
    expect(plugin.executionInterface.wasmPath).toBe(appManifest.entrypoint);
    expect(appManifest.ui.entrypoint).toBe('ui/dist/index.html');
    expect(catalogEntry.repository_url).toBe('https://github.com/nightsky78/Synapp_Apps/');
    expect(catalogEntry.source.repository).toBe('https://github.com/nightsky78/Synapp_Apps/');
    expect(catalogEntry.source.directory).toBe('apps/first-party/calendar-management');
    expect(catalogEntry.package_url).toContain('/calendar-management-v0.1.0/');
    expect(catalogEntry.package_sha256).toMatch(/^[a-f0-9]{64}$/);
    expect(catalogEntry.manifest_sha256).toMatch(/^[a-f0-9]{64}$/);
    expect(catalogEntry.signature.verification_status).toBe('metadata_verified');
    expect(appManifest.resource_limits.memory_bytes).toBeLessThanOrEqual(128 * 1024 * 1024);
    expect(appManifest.resource_limits.timeout_ms).toBeLessThanOrEqual(5000);
  });

  test('TC-SA-CAL-CONTRACT-003: capabilities and host effects match liaison-approved local coverage', async () => {
    const plugin = await readJson<PluginManifest>(path.join(calendarRoot, 'plugin.json'));
    const appManifest = await readJson<AppManifest>(path.join(calendarRoot, 'synapp.app.json'));

    const capabilityState = new Map(appManifest.capabilities.map((item) => [item.capability, item]));
    expect(capabilityState.get('calendar:read')?.required).toBe(true);
    expect(capabilityState.get('calendar:write')?.required).toBe(true);

    for (const optionalCapability of ['calendar:invite', 'calendar:respond', 'calendar:share', 'calendar:delegate', 'calendar:settings', 'calendar:audit']) {
      expect(capabilityState.get(optionalCapability)?.required, `${optionalCapability} must remain optional`).toBe(false);
    }

    expect(plugin.hostEffects.required).toEqual([
      'CalendarStoreRead',
      'CalendarStoreWrite',
      'CalendarStoreDelete',
      'CalendarInviteSend',
      'CalendarInviteRespond',
      'FreeBusyLookup',
      'RoomResourceLookup',
      'ContactLookup',
      'ConferenceLinkCreate',
      'ReminderSchedule',
      'CalendarShareManage',
      'SubscriptionSync',
      'OfflineCacheRead',
      'AuditRead',
      'AuditWrite',
      'NotifyUser',
    ]);

    const workspace = appManifest.ui.contributions[0]?.page_layout_schema?.components?.[0];
    expect(workspace?.type).toBe('CalendarWorkspace');
    expect(workspace?.required_capabilities).toEqual(['calendar:read']);
    expect(workspace?.optional_capabilities).toEqual(['calendar:write', 'calendar:invite', 'calendar:respond', 'calendar:share', 'calendar:delegate', 'calendar:settings', 'calendar:audit']);
  });

  test('TC-SA-CAL-CONTRACT-004: basic event persistence uses generic document host effects', async () => {
    const plugin = await readJson<PluginManifest>(path.join(calendarRoot, 'plugin.json'));
    const appSource = await readFile(path.join(calendarRoot, 'ui', 'src', 'App.jsx'), 'utf8');
    const bridgeSource = await readFile(path.join(calendarRoot, 'ui', 'src', 'bridge.js'), 'utf8');
    const rustSource = await readFile(path.join(calendarRoot, 'src', 'lib.rs'), 'utf8');

    expect(rustSource).toContain('effect_type: "put_document"');
    expect(rustSource).toContain('effect_type: "query_documents"');
    expect(rustSource).toContain('collection: "events".to_string()');
    expect(rustSource).toContain('existing_event or a matching snapshot event is required');
    expect(rustSource).toContain('fn create_event_emits_put_document_for_events_collection');
    expect(rustSource).toContain('fn update_event_emits_put_document_with_complete_merged_document');
    expect(rustSource).toContain('fn list_events_emits_query_documents_for_events_collection');
    expect(rustSource).not.toContain('"Create calendar event in host store"');
    expect(rustSource).not.toContain('"Update calendar event in host store"');

    expect(appSource).toContain('existing_event: selectedEvent');
    expect(bridgeSource).toContain("persistence_status: 'committed'");
    expect(bridgeSource).toContain("type: 'put_document'");
    expect(bridgeSource).toContain("type: 'query_documents'");

    const hostEffectSchema = plugin.definitions.HostEffect;
    expect(JSON.stringify(hostEffectSchema)).toContain('DocumentHostEffect');
    expect(JSON.stringify(plugin.definitions.UpdateEventInput)).toContain('existing_event');
  });
});

async function readJson<T>(filePath: string): Promise<T> {
  return JSON.parse(await readFile(filePath, 'utf8')) as T;
}

function sorted(values: string[]): string[] {
  return [...values].sort((left, right) => left.localeCompare(right));
}