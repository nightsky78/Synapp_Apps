import { expect, test, type Page } from '@playwright/test';
import { createReadStream } from 'node:fs';
import { stat } from 'node:fs/promises';
import { createServer, type Server } from 'node:http';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const mailDistDir = path.join(repoRoot, 'apps', 'first-party', 'mail-client', 'ui', 'dist');
let mailUiUrl: string;
let mailUiServer: Server;

type HostBridgeOptions = {
  permission?: 'none' | 'read' | 'draft' | 'send' | 'organize' | 'accounts' | 'admin';
  accounts?: unknown[];
  failOperations?: string[];
};

test.describe('Mail Client packaged UI', () => {
  test.beforeAll(async () => {
    const serverInfo = await startStaticServer(mailDistDir);
    mailUiServer = serverInfo.server;
    mailUiUrl = serverInfo.url;
  });

  test.afterAll(async () => {
    await new Promise<void>((resolve, reject) => {
      mailUiServer.close((error) => error ? reject(error) : resolve());
    });
  });

  test('TC-SA-MAIL-UI-000: no host bridge does not render sample inbox data', async ({ page }) => {
    await page.route('**/api/v1/mail/accounts', async (route) => {
      await route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify({ accounts: [] }) });
    });

    await page.goto(mailUiUrl);

    await expect(page.getByRole('heading', { name: 'Add your first mail account' })).toBeVisible();
    await expect(page.getByText('Alice Chen')).toHaveCount(0);
    await expect(page.getByText('Synapp Scheduler')).toHaveCount(0);
    await expect(page.getByText('Scheduled send queued for Friday')).toHaveCount(0);
  });

  test('TC-SA-MAIL-UI-006: no host bridge renders real mail API account and message responses', async ({ page }) => {
    const apiCalls: string[] = [];
    const platformCalls: string[] = [];
    const platformDocs = new Map<string, Map<string, unknown>>();
    const mailboxReads: string[] = [];
    const sentSubjects: string[] = ['Real sent IMAP message'];

    await page.addInitScript(() => {
      (window as any).__SYNAPP_ACCESS_TOKEN__ = 'mail-ui-platform-token';
    });

    function docsFor(collection: string) {
      if (!platformDocs.has(collection)) platformDocs.set(collection, new Map());
      return platformDocs.get(collection)!;
    }

    await page.route('**/api/v1/platform/mail-client/documents/**', async (route) => {
      const request = route.request();
      const requestUrl = new URL(request.url());
      const segments = requestUrl.pathname.split('/');
      const collection = segments[segments.indexOf('documents') + 1];
      platformCalls.push(`${request.method()} ${requestUrl.pathname} ${request.headers().authorization || ''}`);
      if (request.method() === 'GET') {
        const filters = JSON.parse(requestUrl.searchParams.get('filter') || '{}') as Record<string, string>;
        const documents = [...docsFor(collection).entries()]
          .map(([doc_id, data]) => ({ doc_id, data: data as Record<string, unknown> }))
          .filter((doc) => Object.entries(filters).every(([key, value]) => String(doc.data[key]) === String(value)));
        await route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify({ documents }) });
        return;
      }
      if (request.method() === 'POST') {
        const body = request.postDataJSON() as { doc_id?: string; data?: unknown };
        docsFor(collection).set(body.doc_id || `doc-${Date.now()}`, body.data || {});
        await route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify({ doc_id: body.doc_id }) });
        return;
      }
      await route.fulfill({ status: 405, body: 'method not allowed' });
    });

    await page.route('**/api/v1/mail/accounts', async (route) => {
      apiCalls.push(route.request().method() + ' ' + new URL(route.request().url()).pathname);
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          accounts: [{
            account_id: 'real-account-1',
            account_label: 'Real mailbox',
            display_name: 'Real User',
            email_address: 'real@example.com',
            provider: 'manual_imap_smtp',
            auth_method: 'password',
            incoming: { host: 'imap.example.com', port: 993, security: 'ssl_tls', username: 'real@example.com' },
            outgoing: { host: 'smtp.example.com', port: 587, security: 'starttls', username: 'real@example.com' },
            credential_state: 'stored',
            incoming_secret_ref: 'mail:real-account-1:password',
            outgoing_secret_ref: 'mail:real-account-1:password',
            status: 'active',
            is_default: true,
            last_test_at: '2026-05-14T09:00:00Z',
            connection_state: { account_state: 'active' },
          }],
        }),
      });
    });
    await page.route('**/api/v1/mail/send', async (route) => {
      apiCalls.push(route.request().method() + ' ' + new URL(route.request().url()).pathname);
      const requestBody = route.request().postDataJSON() as {
        subject?: string;
        to?: string;
        body_text?: string;
      };
      sentSubjects.unshift(requestBody.subject || '(no subject)');
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          message_id: `real-sent-${Date.now()}`,
          accepted_recipients: requestBody.to ? requestBody.to.split(',').map((email) => email.trim()).filter(Boolean) : [],
          queued: false,
        }),
      });
    });
    await page.route('**/api/v1/mail/messages?**', async (route) => {
      const requestUrl = new URL(route.request().url());
      const mailbox = (requestUrl.searchParams.get('mailbox') || 'INBOX').toUpperCase();
      mailboxReads.push(mailbox);
      apiCalls.push(route.request().method() + ' ' + requestUrl.pathname);
      if (mailbox === 'SENT') {
        await route.fulfill({
          status: 502,
          contentType: 'application/json',
          body: JSON.stringify({ error: 'imap fetch close: imap BAD Error in IMAP command FETCH: Invalid messageset' }),
        });
        return;
      }
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          account: { account_id: 'real-account-1', account_label: 'Real mailbox', email_address: 'real@example.com', status: 'active' },
          mailbox,
          messages: [{
                id: 'real-message-1',
                uid: '42',
                message_id: '<real-message-1@example.com>',
                account_id: 'real-account-1',
                mailbox: 'INBOX',
                from: { name: 'Real Sender', email: 'sender@example.com' },
                to: [{ name: 'Real User', email: 'real@example.com' }],
                subject: 'Real IMAP message',
                snippet: 'This message came from the platform mail API.',
                body_text: 'This message came from the platform mail API.',
                body_html: '<p>This message came from the platform mail API.</p>',
                date_iso: '2026-05-14T08:30:00Z',
                is_read: false,
                has_attachments: false,
              }],
        }),
      });
    });

    await page.goto(mailUiUrl);

    await expect(page.getByLabel('Account scope')).toHaveValue('real-account-1');
    await expect(page.getByRole('button', { name: /Email from Real Sender: Real IMAP message/ })).toBeVisible();
    await page.getByRole('button', { name: /Email from Real Sender: Real IMAP message/ }).click();
    await expect(page.getByRole('heading', { name: 'Real IMAP message' })).toBeVisible();
    await expect(page.getByLabel('Email content').getByText('This message came from the platform mail API.')).toBeVisible();
    await page.getByTestId('email-flag-button-real-message-1').click();
    await expect.poll(() => [...docsFor('messages').values()].find((message: any) => message.email_id === 'real-message-1') as { is_read?: boolean; is_flagged?: boolean }).toMatchObject({ is_read: true, is_flagged: true });

    for (const mailboxId of ['inbox', 'drafts', 'sent', 'archive', 'junk', 'trash']) {
      await expect(page.getByTestId(`mailbox-button-${mailboxId}`)).toBeVisible();
    }

    await page.getByTestId('mailbox-button-sent').click();
    await expect(page.getByTestId('mailbox-button-sent')).toHaveClass(/sidebar__folder--active/);
    await expect(page.getByTestId('sync-warning-sent')).toContainText('Sent sync is delayed. Showing saved messages.');
    await expect(page.getByText('No messages', { exact: true })).toBeVisible();

    await page.getByRole('button', { name: 'Compose new email' }).click();
    const compose = page.getByRole('dialog', { name: 'Compose email' });
    await compose.getByPlaceholder(/Add recipients/).fill('recipient@example.com');
    await compose.getByPlaceholder(/Add recipients/).press('Enter');
    await compose.getByLabel('Subject').fill('Real sent from no-host flow');
    await compose.getByLabel('Message body').fill('Validating that sent mail appears in Sent mailbox.');
    await compose.getByTestId('compose-send-button').click();
    await expect(compose).toHaveCount(0);

    await page.getByTestId('mailbox-button-inbox').click();
    await expect(page.getByTestId('mailbox-button-inbox')).toHaveClass(/sidebar__folder--active/);
    await page.getByTestId('mailbox-button-sent').click();
    await expect(page.getByTestId('mailbox-button-sent')).toHaveClass(/sidebar__folder--active/);
    await expect(page.getByTestId('sync-warning-sent')).toContainText('Sent sync is delayed. Showing saved messages.');
    await expect(page.getByRole('button', { name: /Email from Real User: Real sent from no-host flow/ })).toBeVisible();

    expect(apiCalls).toContain('GET /api/v1/mail/accounts');
    expect(apiCalls).toContain('GET /api/v1/mail/messages');
    expect(apiCalls).toContain('POST /api/v1/mail/send');
    expect(platformCalls).toEqual(expect.arrayContaining([
      expect.stringMatching(/^GET \/api\/v1\/platform\/mail-client\/documents\/messages Bearer mail-ui-platform-token$/),
      expect.stringMatching(/^POST \/api\/v1\/platform\/mail-client\/documents\/messages Bearer mail-ui-platform-token$/),
      expect.stringMatching(/^POST \/api\/v1\/platform\/mail-client\/documents\/mailboxes Bearer mail-ui-platform-token$/),
    ]));
    expect([...docsFor('messages').keys()].every((key) => /^[0-9a-f]{8}-[0-9a-f]{4}-5[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i.test(key))).toBe(true);
    expect([...docsFor('messages').values()].some((message: any) => message.email_id === 'real-message-1')).toBe(true);
    expect([...docsFor('messages').values()].some((message: any) => message.subject === 'Real sent from no-host flow' && message.mailbox_id === 'sent')).toBe(true);
    expect(mailboxReads).toContain('SENT');
    await expect(page.getByText('Alice Chen')).toHaveCount(0);
  });

  test('TC-SA-MAIL-UI-001: mailbox read, search, flag, and bulk archive journeys update visible state', async ({ page }) => {
    await openMailUi(page, { permission: 'admin' });

    await expect(page.getByText('Synapp Mail')).toBeVisible();
    await expect(page.getByLabel('Account scope')).toHaveValue('acc-1');
    await expect(page.getByRole('button', { name: /Email from Alice Chen: Q2 Budget Review/ })).toBeVisible();

    await page.getByRole('button', { name: /Email from Alice Chen: Q2 Budget Review/ }).click();
    await expect(page.getByRole('heading', { name: /Q2 Budget Review/ })).toBeVisible();
    await expect(page.getByLabel('Email content').getByText(/please review the attached Q2 budget numbers/i)).toBeVisible();
    await expect(mailCalls(page, 'mark_read')).resolves.toHaveLength(1);

    await page.getByLabel('Email actions').getByRole('button', { name: 'Flag email' }).click();
    await expect(page.getByLabel('Notifications').getByText('Flagged')).toBeVisible();
    await expect(mailCalls(page, 'flag_email')).resolves.toHaveLength(1);

    await page.getByLabel('Search emails').fill('github');
    await expect(page.getByText(/Found\s+1\s+result/)).toBeVisible();
    await expect(page.getByRole('button', { name: /Search result: \[GitHub\]/ })).toBeVisible();
    await page.getByLabel('Clear search').click();

    await page.getByLabel('Select all visible messages').check();
    await page.getByRole('toolbar', { name: 'Bulk message actions' }).getByRole('button', { name: 'Archive' }).click();
    await expect(page.getByText('Archived selected messages')).toBeVisible();
    await expect(mailCalls(page, 'archive_email')).resolves.toHaveLength(1);
  });

  test('TC-SA-MAIL-UI-007: selected messages move between folders and remain visible in the destination', async ({ page }) => {
    await openMailUi(page, { permission: 'admin' });

    await page.getByRole('button', { name: /Email from Alice Chen: Q2 Budget Review/ }).click();
    await page.getByLabel('Select message Q2 Budget Review').check();
    await page.getByTestId('bulk-move-button').click();
    await expect(page.getByTestId('bulk-move-menu')).toBeVisible();
    await page.getByTestId('bulk-move-option-projects').click();

    await expect(page.getByText('Moved locally. Remote sync pending.')).toBeVisible();
    await expect(page.getByRole('button', { name: /Email from Alice Chen: Q2 Budget Review/ })).toHaveCount(0);

    await page.getByTestId('mailbox-button-projects').click();
    const moved = page.getByRole('button', { name: /Email from Alice Chen: Q2 Budget Review/ });
    await expect(moved).toBeVisible();
    await expect(page.getByTestId('email-sync-state-email-1')).toContainText('Sync pending');
    await moved.click();
    await expect(page.getByLabel('Email content').getByTestId('email-sync-state-email-1')).toContainText('remote mailbox sync pending');
    await expect(mailCalls(page, 'move_email')).resolves.toEqual(expect.arrayContaining([
      expect.objectContaining({ payload: expect.objectContaining({ email_ids: ['email-1'], destination_mailbox_id: 'projects' }) }),
    ]));
  });

  test('TC-SA-MAIL-UI-008: filtered bulk move only selects visible messages', async ({ page }) => {
    await openMailUi(page, { permission: 'admin' });

    await page.getByLabel('Message filters').getByRole('button', { name: 'Unread' }).click();
    await page.getByLabel('Select all visible messages').check();
    await page.getByTestId('bulk-move-button').click();
    await page.getByTestId('bulk-move-option-projects').click();

    await expect(mailCalls(page, 'move_email')).resolves.toEqual(expect.arrayContaining([
      expect.objectContaining({ payload: expect.objectContaining({ email_ids: ['email-1', 'email-4'], destination_mailbox_id: 'projects' }) }),
    ]));

    await page.getByLabel('Message filters').getByRole('button', { name: 'All' }).click();
    await expect(page.getByRole('button', { name: /Email from Bob Smith: Sprint planning notes/ })).toBeVisible();
    await expect(page.getByRole('button', { name: /Email from GitHub: \[GitHub\] Pull request review requested/ })).toBeVisible();
    await expect(page.getByRole('button', { name: /Email from Alice Chen: Q2 Budget Review/ })).toHaveCount(0);
  });

  test('TC-SA-MAIL-UI-002: compose validates recipients, saves drafts, sends, and schedules mail', async ({ page }) => {
    await openMailUi(page, { permission: 'admin' });

    await page.getByRole('button', { name: 'Compose new email' }).click();
    const compose = page.getByRole('dialog', { name: 'Compose email' });
    await expect(compose).toBeVisible();

    await compose.getByRole('button', { name: 'Send', exact: true }).click();
    await expect(page.getByText('Add at least one recipient')).toBeVisible();

    await compose.getByPlaceholder(/Add recipients/).fill('bad-address');
    await compose.getByPlaceholder(/Add recipients/).press('Enter');
    await expect(page.getByText('Invalid email address')).toBeVisible();

    await compose.getByPlaceholder(/Add recipients/).fill('friend@example.com');
    await compose.getByPlaceholder(/Add recipients/).press('Enter');
    await compose.getByLabel('Subject').fill('Launch update');
    await compose.getByLabel('Message body').fill('The launch note is ready for review.');
    await compose.locator('.compose-window__footer').getByRole('button', { name: 'Save draft' }).click();
    await expect(page.getByLabel('Notifications').getByText('Draft saved')).toBeVisible();

    await compose.getByTestId('compose-send-button').click();
    await expect(compose).toHaveCount(0);
    await page.getByTestId('mailbox-button-sent').click();
    await expect(page.getByTestId('mailbox-button-sent')).toHaveClass(/sidebar__folder--active/);
    await expect(page.getByTestId('email-item-sent-draft-ui-001')).toBeVisible();
    await expect(page.getByRole('button', { name: /Email from Test User: Launch update/ })).toBeVisible();
    await expect(mailCalls(page, 'draft_email')).resolves.toHaveLength(1);
    await expect(mailCalls(page, 'send_email')).resolves.toHaveLength(1);
    await expect(mailCalls(page, 'read_emails')).resolves.toEqual(expect.arrayContaining([
      expect.objectContaining({ payload: expect.objectContaining({ mailbox_id: 'sent' }) }),
    ]));

    await page.getByRole('button', { name: 'Compose new email' }).click();
    const scheduled = page.getByRole('dialog', { name: 'Compose email' });
    await scheduled.getByPlaceholder(/Add recipients/).fill('future@example.com');
    await scheduled.getByPlaceholder(/Add recipients/).press('Enter');
    await scheduled.getByLabel('Subject').fill('Scheduled follow-up');
    await scheduled.getByLabel('Message body').fill('Send this tomorrow.');
    await scheduled.getByRole('button', { name: 'Schedule send options' }).click();
    await scheduled.getByRole('button', { name: 'Tomorrow noon' }).click();

    await expect(page.getByText('Scheduled for tomorrow noon')).toBeVisible();
    await expect(mailCalls(page, 'schedule_send')).resolves.toHaveLength(1);
  });

  test('TC-SA-MAIL-UI-003: account setup, preferences, and rules settings call app actions', async ({ page }) => {
    await openMailUi(page, { permission: 'admin' });

    await page.getByRole('button', { name: 'Open mail settings' }).click();
    await expect(page.getByRole('heading', { name: 'Accounts' })).toBeVisible();
    await page.getByRole('button', { name: 'Add account' }).click();

    await page.getByLabel('Account label').fill('Personal');
    await page.getByLabel('Display name').fill('Test User');
    await page.getByLabel('Email address').fill('test@example.com');
    await page.getByLabel('IMAP host').fill('imap.example.com');
    await page.getByLabel('SMTP host').fill('smtp.example.com');
    await page.getByRole('button', { name: 'Test connection' }).click();
    await expect(page.getByLabel('Notifications').getByText('Connection test completed', { exact: true })).toBeVisible();
    await expect(page.getByText('passed').first()).toBeVisible();
    await page.getByRole('button', { name: 'Save receive-only' }).click();
    await expect(page.getByText('Account saved as receive-only')).toBeVisible();
    await expect(mailCalls(page, 'complete_account_setup')).resolves.toHaveLength(1);

    await page.getByRole('button', { name: 'Open mail settings' }).click();
    await page.getByRole('button', { name: 'Preferences' }).click();
    await page.getByLabel('Undo send').selectOption('30');
    await page.getByRole('button', { name: 'Save preferences' }).click();
    await expect(page.getByText('Preferences saved')).toBeVisible();

    await page.getByRole('button', { name: 'Rules' }).last().click();
    await page.getByRole('button', { name: 'Create rule' }).click();
    await expect(page.getByText('Rule name and condition value are required')).toBeVisible();
    await page.getByLabel('Rule Name').fill('Move newsletters');
    await page.getByLabel('Value').fill('newsletter@');
    await page.getByLabel('Target Folder').selectOption('projects');
    await page.getByRole('button', { name: 'Create rule' }).click();
    await expect(page.getByText('Rule created')).toBeVisible();
    await expect(page.getByText('Move newsletters')).toBeVisible();
  });

  test('TC-SA-MAIL-UI-004: read-only permission disables send and organize controls without invoking restricted actions', async ({ page }) => {
    await openMailUi(page, { permission: 'read' });

    await expect(page.locator('.perm-badge')).toContainText('Read');
    await page.getByRole('button', { name: /Email from Alice Chen: Q2 Budget Review/ }).click();
    await expect(page.getByRole('button', { name: 'Reply', exact: true })).toBeDisabled();
    await expect(page.getByRole('button', { name: 'Archive email' })).toBeDisabled();
    await expect(page.getByRole('button', { name: 'Delete email' })).toBeDisabled();

    await page.getByLabel('Select all visible messages').check();
    await expect(page.getByRole('toolbar', { name: 'Bulk message actions' }).getByRole('button', { name: 'Archive' })).toBeDisabled();
    await expect(page.getByRole('toolbar', { name: 'Bulk message actions' }).getByRole('button', { name: 'Delete' })).toBeDisabled();

    await page.getByRole('button', { name: 'Compose new email' }).click();
    const compose = page.getByRole('dialog', { name: 'Compose email' });
    await expect(compose.getByRole('button', { name: 'Send', exact: true })).toBeDisabled();
    await expect(compose.locator('.compose-window__footer').getByRole('button', { name: 'Save draft' })).toBeDisabled();
    await expect(mailCalls(page, 'send_email')).resolves.toHaveLength(0);
    await expect(mailCalls(page, 'archive_email')).resolves.toHaveLength(0);
  });

  test('TC-SA-MAIL-UI-005: host action failures surface visible error toasts', async ({ page }) => {
    await openMailUi(page, { permission: 'admin', failOperations: ['search_emails'] });

    await page.getByLabel('Search emails').fill('budget');
    await expect(page.getByText('Search failed: forced search_emails failure')).toBeVisible();
  });

});

async function openMailUi(page: Page, options: HostBridgeOptions = {}) {
  await page.addInitScript((init) => {
    const now = 1_800_000_000;
    const permission = init.permission ?? 'admin';
    const failures = new Set(init.failOperations ?? []);
    const accounts = init.accounts ?? [{
      account_id: 'acc-1',
      display_name: 'Test User',
      email_address: 'test@example.com',
      provider: 'manual_imap_smtp',
      account_label: 'Work Mail',
      enabled: true,
      is_default: true,
      sync_interval_minutes: 5,
      color: '#2563eb',
      connection_state: 'connected',
      status: 'active',
      credential_state: 'stored',
      last_sync_at: now - 300,
      unread_count: 2,
    }];
    const mailboxes = [
      { mailbox_id: 'inbox', account_id: 'acc-1', name: 'Inbox', kind: 'inbox', unread_count: 2, total_count: 4, favorite: true },
      { mailbox_id: 'drafts', account_id: 'acc-1', name: 'Drafts', kind: 'drafts', unread_count: 0, total_count: 1, favorite: false },
      { mailbox_id: 'sent', account_id: 'acc-1', name: 'Sent', kind: 'sent', unread_count: 0, total_count: 8, favorite: false },
      { mailbox_id: 'archive', account_id: 'acc-1', name: 'Archive', kind: 'archive', unread_count: 0, total_count: 12, favorite: false },
      { mailbox_id: 'projects', account_id: 'acc-1', name: 'Projects', kind: 'custom', unread_count: 0, total_count: 3, favorite: true },
    ];
    const emails = [
      makeEmail(1, 'alice.chen@company.com', 'Alice Chen', 'Q2 Budget Review', 'Please review the attached Q2 budget numbers before our Friday meeting.', now - 3600, false, true, 'high'),
      makeEmail(2, 'bob.smith@company.com', 'Bob Smith', 'Sprint planning notes', 'The analytics feature can move to next sprint.', now - 7200, true, false, 'normal'),
      makeEmail(3, 'noreply@github.com', 'GitHub', '[GitHub] Pull request review requested', 'Review requested on PR #142.', now - 86400, true, false, 'normal'),
      makeEmail(4, 'newsletter@example.com', 'Newsletter', 'Weekly digest', 'A short digest of product updates.', now - 172800, false, false, 'low'),
    ];
    const rules = [];
    const calls = [];
    const draftStore = new Map();

    function makeEmail(id, fromEmail, fromName, subject, preview, receivedAt, isRead, hasAttachments, importance) {
      return {
        email_id: `email-${id}`,
        account_id: 'acc-1',
        mailbox_id: 'inbox',
        thread_id: `thread-${id}`,
        from: { email: fromEmail, display_name: fromName },
        to: [{ email: 'test@example.com', display_name: 'Test User' }],
        cc: [],
        bcc: [],
        subject,
        preview,
        body_html: `<p>${preview}</p>`,
        body_text: preview,
        received_at: receivedAt,
        sent_at: 0,
        is_read: isRead,
        is_flagged: false,
        has_attachments: hasAttachments,
        importance,
        categories: [],
        attachments: hasAttachments ? [{ attachment_id: `att-${id}`, file_name: 'budget.pdf', content_type: 'application/pdf', size_bytes: 204800 }] : [],
      };
    }

    function makeSentEmail(payload, draft) {
      const sentAt = Math.floor(Date.now() / 1000);
      const account = accounts.find((item) => item.account_id === (payload.account_id || draft.account_id)) || accounts[0] || {};
      const subject = draft.subject || '(no subject)';
      const preview = draft.body_text || draft.body_html || '';
      return {
        email_id: `sent-${payload.draft_id || sentAt}`,
        account_id: payload.account_id || draft.account_id || 'acc-1',
        mailbox_id: 'sent',
        thread_id: `thread-${payload.draft_id || sentAt}`,
        from: { email: account.email_address || 'test@example.com', display_name: account.display_name || account.account_label || 'Test User' },
        to: (draft.to || []).map((recipient) => ({ email: recipient.email || recipient, display_name: recipient.display_name || recipient.email || recipient })),
        cc: (draft.cc || []).map((recipient) => ({ email: recipient.email || recipient, display_name: recipient.display_name || recipient.email || recipient })),
        bcc: (draft.bcc || []).map((recipient) => ({ email: recipient.email || recipient, display_name: recipient.display_name || recipient.email || recipient })),
        subject,
        preview,
        body_html: draft.body_html || `<p>${preview}</p>`,
        body_text: draft.body_text || preview,
        received_at: sentAt,
        sent_at: sentAt,
        is_read: true,
        is_flagged: false,
        has_attachments: Boolean(draft.attachments?.length),
        importance: draft.importance || 'normal',
        categories: [],
        attachments: draft.attachments || [],
      };
    }

    function ok(operation, data) {
      return { ok: { operation, status: 'accepted', data, host_effects: [], warnings: [] } };
    }

    function mailboxSnapshot(mailboxId = 'inbox') {
      const messages = emails.filter((email) => email.mailbox_id === mailboxId);
      return { messages, total_count: messages.length, unread_count: messages.filter((email) => !email.is_read).length };
    }

    function failIfForced(operation) {
      if (!failures.has(operation)) return null;
      return {
        err: {
          code: 'ForcedFailure',
          message: `forced ${operation} failure`,
        },
      };
    }

    window.__mailCalls = calls;
    window.__synapp = {
      getContext() {
        return {
          user_id: 'mail-ui-test-user',
          permission,
          available_effects: ['AccountCredentialRead', 'AccountCredentialWrite', 'ProviderCapabilityRead', 'ImapFetch', 'ImapSync', 'SmtpSend', 'ContactLookup', 'NotifyUser', 'ScheduleJob', 'CancelJob', 'OAuthConnect', 'OAuthDisconnect', 'StorePlatformSecret', 'MailStoreRead', 'MailStoreWrite', 'MailStoreDelete'],
          agent_id: null,
          request_id: 'mail-ui-test-request',
        };
      },
      async invoke(operation, payload) {
        calls.push({ operation, payload });
        const forced = failIfForced(operation);
        if (forced) return forced;
        switch (operation) {
          case 'list_accounts': return ok(operation, { snapshot: accounts, total_count: accounts.length, refresh_requested: false });
          case 'list_mailboxes': return ok(operation, { snapshot: mailboxes, total_count: mailboxes.length, refresh_requested: false });
          case 'read_emails': return ok(operation, { snapshot: mailboxSnapshot(payload.mailbox_id), needs_host_refresh: false });
          case 'search_emails': {
            const query = String(payload.query ?? '').toLowerCase();
            const results = emails.filter((email) => `${email.subject} ${email.preview} ${email.from.email}`.toLowerCase().includes(query));
            return ok(operation, { query: payload.query, snapshot: { results, total_count: results.length }, needs_host_refresh: false });
          }
          case 'mark_read':
            emails.forEach((email) => { if (payload.email_ids?.includes(email.email_id)) email.is_read = payload.read; });
            return ok(operation, { mutation: 'mark_read', resource_ids: payload.email_ids ?? [], applied: true });
          case 'flag_email':
            emails.forEach((email) => { if (payload.email_ids?.includes(email.email_id)) email.is_flagged = payload.flagged; });
            return ok(operation, { mutation: 'flag_email', resource_ids: payload.email_ids ?? [], applied: true });
          case 'move_email':
            emails.forEach((email) => {
              if (payload.email_ids?.includes(email.email_id)) {
                email.mailbox_id = payload.destination_mailbox_id;
                email.remote_sync_state = 'pending_remote_move';
              }
            });
            return ok(operation, { mutation: 'move_email', resource_ids: payload.email_ids ?? [], destination_mailbox_id: payload.destination_mailbox_id, applied: true, sync_state: 'pending_remote_move' });
          case 'archive_email':
            return ok(operation, { mutation: 'archive_email', resource_ids: payload.email_ids ?? [], applied: true });
          case 'delete_email':
            return ok(operation, { mutation: 'delete_email', resource_ids: payload.email_ids ?? [], applied: true });
          case 'draft_email': {
            const draftId = payload.draft?.draft_id || 'draft-ui-001';
            const draft = { ...payload.draft, draft_id: draftId };
            draftStore.set(draftId, draft);
            return ok(operation, { draft, suggested_draft_id: draftId });
          }
          case 'update_draft': {
            const draftId = payload.draft?.draft_id || 'draft-ui-001';
            const draft = { ...payload.draft, draft_id: draftId };
            draftStore.set(draftId, draft);
            return ok(operation, { draft, suggested_draft_id: draftId });
          }
          case 'send_email': {
            const draft = draftStore.get(payload.draft_id) || payload.draft || payload;
            const sentMessage = makeSentEmail(payload, draft);
            emails.unshift(sentMessage);
            draftStore.delete(payload.draft_id);
            return ok(operation, { draft_id: payload.draft_id, account_id: payload.account_id, undo_window_seconds: 10, notify: true, message_id: sentMessage.email_id });
          }
          case 'schedule_send': return ok(operation, { job_id: 'job-ui-001', draft_id: payload.draft_id, account_id: payload.account_id, scheduled_for: payload.scheduled_for });
          case 'discard_draft':
            draftStore.delete(payload.draft_id);
            return ok(operation, { mutation: 'discard_draft', resource_ids: [payload.draft_id], applied: true });
          case 'get_provider_capabilities': return ok(operation, { providers: [], manual_imap_smtp: { enabled: true }, policy: { allow_receive_only: true } });
          case 'validate_account_setup': return ok(operation, { valid: true, normalized_account: payload.account, field_errors: [], warnings: [], next_required_action: 'test_connection' });
          case 'plan_connection_test': return ok(operation, { account_id: 'acc-new', test_id: 'test-ui-001', requires_host_execution: true, steps: ['imap_auth', 'imap_mailbox_discovery', 'smtp_auth', 'smtp_send_capability', 'folder_mapping'] });
          case 'complete_account_setup': return ok(operation, { account: { account_id: 'acc-new', ...payload.account, connection_state: payload.save_mode === 'receive_only' ? 'receive_only' : 'connected' }, status: payload.save_mode, requires_reconnect: false, requires_sync: true });
          case 'list_identities': return ok(operation, { identities: [{ identity_id: 'id-1', account_id: 'acc-1', kind: 'primary', display_name: 'Test User', email_address: 'test@example.com', enabled: true, verification_state: 'allowed' }], signatures: [{ signature_id: 'sig-1', account_id: 'acc-1', label: 'Default', body_text: 'Regards', enabled: true }], needs_host_refresh: false });
          case 'get_user_preferences': return ok(operation, { preferences: { reading: { preview_pane: 'right', thread_view: true, focused_inbox: true, page_size: 50 }, compose: { undo_send_delay_seconds: 10, default_format: 'html' }, notifications: { enabled: true, per_account: {} }, sync_defaults: { interval_minutes: 15, offline_cache: false } }, defaults_applied: true });
          case 'save_user_preferences': return ok(operation, { preferences: payload.preferences, saved: true });
          case 'list_rules': return ok(operation, { snapshot: rules, total_count: rules.length, refresh_requested: false });
          case 'create_rule':
            rules.push(payload.rule);
            return ok(operation, { rule_id: payload.rule?.rule_id, name: payload.rule?.name, saved: true });
          case 'delete_rule': return ok(operation, { mutation: 'delete_rule', resource_ids: [payload.rule_id], applied: true });
          case 'remove_account': return ok(operation, { account_id: payload.account_id, status: 'removed_pending_cleanup', cleanup_planned: ['local_cache', 'scheduled_sends'] });
          default: return { err: { code: 'UnknownOperation', message: `Unknown operation: ${operation}` } };
        }
      },
    };
  }, options);

  await page.goto(mailUiUrl);
}

async function mailCalls(page: Page, operation: string) {
  return page.evaluate((op) => (window as any).__mailCalls.filter((call: { operation: string }) => call.operation === op), operation);
}

async function startStaticServer(rootDir: string): Promise<{ server: Server; url: string }> {
  const server = createServer(async (request, response) => {
    try {
      const requestUrl = new URL(request.url ?? '/', 'http://127.0.0.1');
      const relativePath = decodeURIComponent(requestUrl.pathname === '/' ? '/index.html' : requestUrl.pathname);
      const filePath = path.normalize(path.join(rootDir, relativePath));

      if (!filePath.startsWith(rootDir)) {
        response.writeHead(403);
        response.end('Forbidden');
        return;
      }

      const fileStat = await stat(filePath);
      if (!fileStat.isFile()) {
        response.writeHead(404);
        response.end('Not found');
        return;
      }

      response.writeHead(200, { 'Content-Type': contentType(filePath) });
      createReadStream(filePath).pipe(response);
    } catch {
      response.writeHead(404);
      response.end('Not found');
    }
  });

  await new Promise<void>((resolve) => server.listen(0, '127.0.0.1', resolve));
  const address = server.address();
  if (!address || typeof address === 'string') throw new Error('Static server did not bind to a TCP port');
  return { server, url: `http://127.0.0.1:${address.port}/` };
}

function contentType(filePath: string): string {
  if (filePath.endsWith('.html')) return 'text/html; charset=utf-8';
  if (filePath.endsWith('.js')) return 'text/javascript; charset=utf-8';
  if (filePath.endsWith('.css')) return 'text/css; charset=utf-8';
  if (filePath.endsWith('.svg')) return 'image/svg+xml';
  return 'application/octet-stream';
}