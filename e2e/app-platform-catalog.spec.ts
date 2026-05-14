import { expect, test, type Page } from '@playwright/test';
import { readdir, readFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

type ManifestCapability = {
  capability: string;
  label: string;
  description: string;
  required: boolean;
  risk: 'low' | 'medium' | 'high' | 'critical';
};

type AppCatalogEntry = {
  app_id: string;
  name: string;
  summary: string;
  description: string;
  version: string;
  publisher: string;
  category: string;
  capabilities: Array<ManifestCapability & { granted: boolean }>;
  installed: boolean;
  enabled: boolean;
  verification_status: 'pending' | 'passed' | 'failed' | 'unavailable';
};

type InstalledApp = {
  app_id: string;
  name: string;
  version: string;
  enabled: boolean;
  publisher?: string;
  update_available: boolean;
  latest_version?: string;
  capabilities: AppCatalogEntry['capabilities'];
};

type MockAppPlatformController = {
  installCalls: string[];
  uninstallCalls: string[];
  grantPayloads: Array<{ app_id: string; capabilities: string[] }>;
};

type CatalogEntryFile = {
  app_id: string;
  name: string;
  summary: string;
  description: string;
  version: string;
  publisher: string;
  category: string;
  signature?: { verification_status?: string };
  manifest: {
    capabilities: ManifestCapability[];
  };
};

type CatalogIndex = {
  apps: Array<{
    app_id: string;
    catalog_entry: string;
  }>;
};

type PackageManifest = {
  app_id: string;
  name: string;
};

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

test.describe('Synapp Apps app-platform catalog visibility', () => {
  test('TC-SA-CAT-001: every first-party app manifest is indexed in the catalog', async () => {
    const firstPartyApps = await loadFirstPartyApps();
    const indexedApps = new Set((await loadCatalogIndex()).apps.map((app) => app.app_id));

    for (const app of firstPartyApps) {
      expect(indexedApps, `${app.name} (${app.app_id}) must be present in catalog/index.v1.json`).toContain(app.app_id);
    }
  });

  test('TC-SA-CAT-002: every indexed app is visible on the App Platform catalog page', async ({ page }) => {
    const catalog = await loadCatalogEntries();
    await mockAppPlatform(page, catalog);

    await page.goto('/admin');
    await page.getByTestId('admin-nav-app-platform').click();

    await expect(page.getByTestId('app-platform-tab-catalog')).toContainText(`Catalog (${catalog.length})`);

    for (const app of catalog) {
      const card = page.getByTestId(`app-card-${app.app_id}`);
      await expect(card, `${app.name} should render as app-card-${app.app_id}`).toBeVisible();
      await expect(card).toContainText(app.name);
      await expect(card).toContainText(app.summary);
      await expect(card).toContainText(`v${app.version}`);
    }
  });

  test('TC-SA-CAT-003: Calendar Management install, capability grants, uninstall, and reinstall lifecycle', async ({ page }) => {
    const catalog = await loadCatalogEntries();
    const calendar = findCatalogApp(catalog, 'calendar-management');
    const platform = await mockAppPlatform(page, catalog);

    await page.goto('/admin');
    await page.getByTestId('admin-nav-app-platform').click();

    await expect(page.getByTestId('app-card-calendar-management')).toContainText(calendar.name);
    await page.getByTestId('app-install-button-calendar-management').click();
    await expect(page.getByTestId('app-install-modal')).toContainText('Read calendars');
    await page.getByTestId('app-install-confirm').click();

    await expect(page.getByTestId('installed-app-row-calendar-management')).toContainText(calendar.name);
    expect(platform.installCalls).toEqual(['calendar-management']);

    await page.getByTestId('app-review-button-calendar-management').click();
    await expect(page.getByTestId('app-capability-locked-calendar:read')).toBeVisible();
    await expect(page.getByTestId('app-capability-toggle-calendar:write')).not.toBeChecked();
    await expect(page.getByTestId('app-capability-toggle-calendar:settings')).not.toBeChecked();
    await expect(page.getByTestId('app-capability-save')).toBeDisabled();

    await page.getByTestId('app-capability-toggle-calendar:write').check();
    await page.getByTestId('app-capability-toggle-calendar:settings').check();
    await page.getByTestId('app-capability-save').click();

    await expect(page.getByTestId('app-capability-save')).toBeDisabled();
    expect(platform.grantPayloads.at(-1)).toEqual({
      app_id: 'calendar-management',
      capabilities: ['calendar:read', 'calendar:write', 'calendar:settings'],
    });
    await expect(page.getByTestId('app-capability-toggle-calendar:write')).toBeChecked();
    await expect(page.getByTestId('app-capability-toggle-calendar:settings')).toBeChecked();

    await page.getByTestId('app-capability-toggle-calendar:settings').uncheck();
    await page.getByTestId('app-capability-save').click();

    await expect(page.getByTestId('app-capability-save')).toBeDisabled();
    expect(platform.grantPayloads.at(-1)).toEqual({
      app_id: 'calendar-management',
      capabilities: ['calendar:read', 'calendar:write'],
    });
    await expect(page.getByTestId('app-capability-toggle-calendar:write')).toBeChecked();
    await expect(page.getByTestId('app-capability-toggle-calendar:settings')).not.toBeChecked();

    await page.getByTestId('app-platform-tab-installed').click();
    await page.getByTestId('app-uninstall-button-calendar-management').click();
    await expect(page.getByTestId('app-uninstall-modal')).toContainText('Uninstall Calendar Management');
    await page.getByTestId('app-uninstall-confirm').click();

    await expect(page.getByTestId('installed-app-row-calendar-management')).toHaveCount(0);
    await expect(page.getByTestId('app-install-button-calendar-management')).toBeVisible();
    expect(platform.uninstallCalls).toEqual(['calendar-management']);

    await page.getByTestId('app-install-button-calendar-management').click();
    await page.getByTestId('app-install-confirm').click();

    await expect(page.getByTestId('installed-app-row-calendar-management')).toContainText(calendar.name);
    expect(platform.installCalls).toEqual(['calendar-management', 'calendar-management']);

    await page.getByTestId('app-review-button-calendar-management').click();
    await expect(page.getByTestId('app-capability-locked-calendar:read')).toBeVisible();
    await expect(page.getByTestId('app-capability-toggle-calendar:write')).not.toBeChecked();
    await expect(page.getByTestId('app-capability-toggle-calendar:settings')).not.toBeChecked();
  });
});

async function loadFirstPartyApps(): Promise<PackageManifest[]> {
  const firstPartyDir = path.join(repoRoot, 'apps', 'first-party');
  const entries = await readdir(firstPartyDir, { withFileTypes: true });
  const manifests = await Promise.all(entries
    .filter((entry) => entry.isDirectory())
    .map(async (entry) => readJson<PackageManifest>(path.join(firstPartyDir, entry.name, 'synapp.app.json'))));

  return manifests.sort((left, right) => left.app_id.localeCompare(right.app_id));
}

async function loadCatalogIndex(): Promise<CatalogIndex> {
  return readJson<CatalogIndex>(path.join(repoRoot, 'catalog', 'index.v1.json'));
}

async function loadCatalogEntries(): Promise<AppCatalogEntry[]> {
  const index = await loadCatalogIndex();
  const entries = await Promise.all(index.apps.map(async (item) => {
    const entry = await readJson<CatalogEntryFile>(path.join(repoRoot, item.catalog_entry));
    return toAppCatalogEntry(entry);
  }));

  return entries.sort((left, right) => left.app_id.localeCompare(right.app_id));
}

function toAppCatalogEntry(entry: CatalogEntryFile): AppCatalogEntry {
  return {
    app_id: entry.app_id,
    name: entry.name,
    summary: entry.summary,
    description: entry.description,
    version: entry.version,
    publisher: entry.publisher,
    category: entry.category,
    installed: false,
    enabled: false,
    verification_status: verificationStatus(entry.signature?.verification_status),
    capabilities: entry.manifest.capabilities.map((capability) => ({
      ...capability,
      granted: capability.required,
    })),
  };
}

function findCatalogApp(catalog: AppCatalogEntry[], appId: string): AppCatalogEntry {
  const app = catalog.find((candidate) => candidate.app_id === appId);
  if (!app) throw new Error(`${appId} is missing from catalog/index.v1.json`);
  return app;
}

function verificationStatus(status: string | undefined): AppCatalogEntry['verification_status'] {
  if (status === 'verified') return 'passed';
  if (status === 'unverified') return 'failed';
  return 'unavailable';
}

async function mockAppPlatform(page: Page, catalog: AppCatalogEntry[]): Promise<MockAppPlatformController> {
  const installedIds = new Set<string>();
  const enabledIds = new Set<string>();
  const optionalGrants = new Map<string, Set<string>>();
  const controller: MockAppPlatformController = {
    installCalls: [],
    uninstallCalls: [],
    grantPayloads: [],
  };

  const appById = new Map(catalog.map((app) => [app.app_id, app]));

  function grantedCapabilities(app: AppCatalogEntry): AppCatalogEntry['capabilities'] {
    const grants = optionalGrants.get(app.app_id) ?? new Set<string>();
    return app.capabilities.map((capability) => ({
      ...capability,
      granted: capability.required || grants.has(capability.capability),
    }));
  }

  function catalogPayload(): AppCatalogEntry[] {
    return catalog.map((app) => ({
      ...app,
      installed: installedIds.has(app.app_id),
      enabled: enabledIds.has(app.app_id),
      capabilities: grantedCapabilities(app),
    }));
  }

  function installedPayload(): InstalledApp[] {
    return catalog
      .filter((app) => installedIds.has(app.app_id))
      .map((app) => ({
        app_id: app.app_id,
        name: app.name,
        version: app.version,
        enabled: enabledIds.has(app.app_id),
        publisher: app.publisher,
        update_available: false,
        capabilities: grantedCapabilities(app),
      }));
  }

  function reviewPayload(app: AppCatalogEntry) {
    return {
      app_id: app.app_id,
      app_name: app.name,
      version: app.version,
      capabilities: grantedCapabilities(app),
    };
  }

  await page.route('**/api/auth/refresh', async (route) => {
    await route.fulfill({
      contentType: 'application/json',
      body: JSON.stringify({ access_token: createJwt(), expires_in: 3600 }),
    });
  });

  await page.route('**/api/auth/me', async (route) => {
    await route.fulfill({
      contentType: 'application/json',
      body: JSON.stringify({
        user: {
          id: 'catalog-test-admin',
          username: 'catalog-admin',
          email: 'catalog-admin@example.invalid',
          display_name: 'Catalog Admin',
          enabled: true,
          groups: [],
          roles: ['admin'],
          mfa_enabled: false,
          quota_used_bytes: 0,
          quota_total_bytes: 1_073_741_824,
        },
      }),
    });
  });

  await page.route('**/api/app-catalog/v1/apps', async (route) => {
    await route.fulfill({ contentType: 'application/json', body: JSON.stringify({ apps: catalogPayload() }) });
  });

  await page.route('**/api/admin/apps/installed', async (route) => {
    await route.fulfill({ contentType: 'application/json', body: JSON.stringify({ apps: installedPayload() }) });
  });

  await page.route('**/api/v1/apps/enabled', async (route) => {
    const apps = catalogPayload().filter((app) => app.installed && app.enabled);
    await route.fulfill({ contentType: 'application/json', body: JSON.stringify({ apps }) });
  });

  await page.route('**/api/admin/apps/*/capability-review', async (route) => {
    const appId = appIdFromAdminAppsUrl(route.request().url());
    const app = appById.get(appId);
    if (!app) {
      await route.fulfill({ status: 404, body: 'not found' });
      return;
    }

    await route.fulfill({ contentType: 'application/json', body: JSON.stringify(reviewPayload(app)) });
  });

  await page.route('**/api/admin/apps/*/capability-grants', async (route) => {
    const appId = appIdFromAdminAppsUrl(route.request().url());
    const app = appById.get(appId);
    if (!app) {
      await route.fulfill({ status: 404, body: 'not found' });
      return;
    }

    const body = route.request().postDataJSON() as { capabilities: string[] };
    const knownCapabilities = new Set(app.capabilities.map((capability) => capability.capability));
    const requiredCapabilities = app.capabilities.filter((capability) => capability.required).map((capability) => capability.capability);
    const hasUnknownCapability = body.capabilities.some((capability) => !knownCapabilities.has(capability));
    const hasAllRequiredCapabilities = requiredCapabilities.every((capability) => body.capabilities.includes(capability));

    if (hasUnknownCapability || !hasAllRequiredCapabilities) {
      await route.fulfill({ status: 400, contentType: 'application/json', body: JSON.stringify({ error: 'invalid capability grant request' }) });
      return;
    }

    const optional = new Set(body.capabilities.filter((capability) => !requiredCapabilities.includes(capability)));
    optionalGrants.set(appId, optional);
    controller.grantPayloads.push({ app_id: appId, capabilities: body.capabilities });
    await route.fulfill({ contentType: 'application/json', body: JSON.stringify(reviewPayload(app)) });
  });

  await page.route('**/api/admin/apps/*/install', async (route) => {
    const appId = appIdFromAdminAppsUrl(route.request().url());
    const app = appById.get(appId);
    if (!app) {
      await route.fulfill({ status: 404, body: 'not found' });
      return;
    }

    controller.installCalls.push(appId);
    installedIds.add(appId);
    enabledIds.add(appId);
    optionalGrants.set(appId, new Set());
    const installedApp = installedPayload().find((candidate) => candidate.app_id === appId);
    await route.fulfill({ contentType: 'application/json', body: JSON.stringify({ app: installedApp }) });
  });

  await page.route('**/api/admin/apps/*', async (route) => {
    if (route.request().method() !== 'DELETE') {
      await route.fallback();
      return;
    }

    const appId = appIdFromAdminAppsUrl(route.request().url());
    controller.uninstallCalls.push(appId);
    installedIds.delete(appId);
    enabledIds.delete(appId);
    optionalGrants.delete(appId);
    await route.fulfill({ status: 204, body: '' });
  });

  await page.route('**/api/admin/users', async (route) => {
    await route.fulfill({ contentType: 'application/json', body: JSON.stringify({ users: [] }) });
  });

  await page.route('**/api/admin/trash/settings', async (route) => {
    await route.fulfill({ contentType: 'application/json', body: JSON.stringify({ min_retention_days: 30, max_retention_days: 30 }) });
  });

  await page.route('**/api/admin/system/llm', async (route) => {
    await route.fulfill({
      contentType: 'application/json',
      body: JSON.stringify({
        interface_url: 'http://localhost:11434',
        default_model: 'local-model',
        embedding_interface_url: 'http://localhost:11434',
        embedding_model: 'local-embedding',
        embedding_dimensions: 1536,
        vision_enabled: false,
        has_token: false,
        token_last4: '',
      }),
    });
  });

  await page.route('**/api/admin/apps/settings/schemas', async (route) => {
    await route.fulfill({ contentType: 'application/json', body: JSON.stringify({ schemas: [] }) });
  });

  await page.route('**/api/admin/system/settings', async (route) => {
    await route.fulfill({ contentType: 'application/json', body: JSON.stringify({ maintenance_mode: false }) });
  });

  await page.route('**/api/user/agents', async (route) => {
    await route.fulfill({ contentType: 'application/json', body: JSON.stringify({ agents: [] }) });
  });

  await page.route('**/api/auth/mfa/status', async (route) => {
    await route.fulfill({
      contentType: 'application/json',
      body: JSON.stringify({ mfa_enabled: false, totp_configured: false, webauthn_key_count: 0, device_token_count: 0 }),
    });
  });

  await page.route('**/api/auth/device-tokens', async (route) => {
    await route.fulfill({ contentType: 'application/json', body: JSON.stringify({ tokens: [] }) });
  });

  await page.route('**/api/v1/notifications', async (route) => {
    await route.fulfill({ contentType: 'application/json', body: JSON.stringify([]) });
  });

  await page.route('**/api/v1/events/stream?*', async (route) => {
    await route.fulfill({ status: 200, contentType: 'text/event-stream', body: '' });
  });

  return controller;
}

async function readJson<T>(filePath: string): Promise<T> {
  return JSON.parse(await readFile(filePath, 'utf8')) as T;
}

function appIdFromAdminAppsUrl(url: string): string {
  const segments = new URL(url).pathname.split('/').filter(Boolean);
  return segments[3];
}

function createJwt(): string {
  const header = encodeBase64Url(JSON.stringify({ alg: 'none', typ: 'JWT' }));
  const payload = encodeBase64Url(JSON.stringify({ sub: 'catalog-test-admin', exp: 4_102_444_800 }));
  return `${header}.${payload}.signature`;
}

function encodeBase64Url(value: string): string {
  return Buffer.from(value, 'utf8').toString('base64url');
}