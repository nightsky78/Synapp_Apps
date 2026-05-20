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
  ui_schemas?: {
    main?: {
      schema_type?: string;
      page_layout_schema?: {
        layout?: string;
        components?: Array<{
          kind?: string;
          id?: string;
          initial_view?: string;
          supported_views?: string[];
        }>;
      };
    };
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

    const workspace = appManifest.ui_schemas?.main?.page_layout_schema?.components?.[0];
    expect(appManifest.ui_schemas?.main?.schema_type).toBe('page_layout_schema');
    expect((appManifest.ui_schemas?.main as any)?.resource?.uri).toBe('app://calendar-management/ui/main');
    expect((appManifest.ui_schemas?.main as any)?.resource?.mime_type).toBe('application/vnd.synapp.page-layout+json');
    expect(appManifest.ui_schemas?.main?.page_layout_schema?.layout).toBe('workspace');
    expect(workspace?.kind).toBe('calendar.workspace.v1');
    expect(workspace?.id).toBe('calendar.workspace');
    expect(workspace?.initial_view).toBe('week');
    expect(workspace?.supported_views).toEqual(['day', 'week', 'month', 'agenda']);
    const affordances = (workspace as any)?.affordances;
    expect(affordances?.find_meeting_times?.tool).toBe('suggest_meeting_times');
    expect(affordances?.edit_occurrence_vs_series?.tool).toBe('update_event');
    expect(affordances?.edit_occurrence_vs_series?.scope_values).toEqual(['occurrence', 'series']);
    expect(affordances?.rsvp_with_comment_and_notify?.tool).toBe('respond_to_invitation');
    expect(affordances?.rsvp_with_comment_and_notify?.comment_field).toBe('message');
    expect(affordances?.rsvp_with_comment_and_notify?.notify_field).toBe('send_response');
  });

  test('TC-SA-CAL-CONTRACT-004: event persistence contract targets the platform calendar API', async () => {
    const plugin = await readJson<PluginManifest>(path.join(calendarRoot, 'plugin.json'));
    const appSource = await readFile(path.join(calendarRoot, 'ui', 'src', 'App.jsx'), 'utf8');
    const bridgeSource = await readFile(path.join(calendarRoot, 'ui', 'src', 'bridge.js'), 'utf8');
    const rustSource = await readFile(path.join(calendarRoot, 'src', 'lib.rs'), 'utf8');
    const appManifest = await readJson<AppManifest>(path.join(calendarRoot, 'synapp.app.json'));

    expect(appManifest.description).toContain('platform calendar API');
    expect(appManifest.ui_schemas?.main?.page_layout_schema?.components?.[0]?.kind).toBe('calendar.workspace.v1');
    expect(rustSource).toContain('existing_event or a matching snapshot event is required');

    expect(appSource).toContain('existing_event: selectedEvent');
    expect(bridgeSource).toContain('window.__synapp');

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