import { test, expect } from '@playwright/test';

const nodeConfig = `
---
keypair: ae47e1cvAPimcM7dQFwUQSNXdVfhUk3q1Hua4W7AG991mT5dhYEDoixFU9WPQV7CKZDU1qTbjxk3VCZMSAVugL47Lp
public_key: peHozo7MdJBzjF11nYXnbojD1QLsCRv2w33jDW3fXrmgtc
name: web
id: 12D3KooWSdMxrVD1NtzjQuHyAPM9W16EqCF4wj1X1KaXRegkfdcY
listen_addresses: ~
addresses: ~
cells:
  - inline:
      public_key: pe2N17w6mbsfhZs8TUAGc1DtxaiX6kTottJc6eLcf3hx85
      keypair: ae5LTr9mihQZjMm1oPWicSCuiSQQo12G2PN8pZzm1MQgvgdbWLjaQAbPwaipvGHZ6DeLmRCKReMM7RX8iob4AGgmYR
      name: cell
      id: 12D3KooWBBNHgEMK4Zy4H1xtnsMLzh3RCr9PVVyNGCTfibowbtr1
      nodes:
        - node:
            public_key: pe8Fij7YgKsgoBmp9JBVd196XH1SgWMZVddhmKqBFwBVS6
            name: server
            id: 12D3KooWH55trgG34b4gBheip6NLutc7Vmj9PFa7bJ8MDAQq5SA2
            addresses:
              p2p:
                - /ip4/127.0.0.1/tcp/3359
                - /ip4/127.0.0.1/tcp/3459/ws
              http:
                - "http://127.0.0.1:8059"
          roles:
            - 1
            - 2
            - 3
        - node:
            public_key: peHozo7MdJBzjF11nYXnbojD1QLsCRv2w33jDW3fXrmgtc
            name: web
            id: 12D3KooWSdMxrVD1NtzjQuHyAPM9W16EqCF4wj1X1KaXRegkfdcY
            addresses:
              p2p:
                - /ip4/127.0.0.1/tcp/3355
                - /ip4/127.0.0.1/tcp/3455/ws
              http:
                - "http://127.0.0.1:8055"
          roles: []
      apps: []
store: ~
chain: ~
`;

test.beforeEach(async ({ page }) => {
  page.on('console', msg => console.log('Browser console: ' + msg.text()));

  await page.goto('http://127.0.0.1:8080');
});

test.describe('Exocore', () => {
  test.beforeEach(async ({ page }) => {
    const config = page.locator('#config');
    if (config) {
      await page.locator('#config').fill(nodeConfig);
      await page.locator('#config-save').click();
    }

    await page.waitForSelector('#input-text');
    await page.waitForFunction(() => !document.querySelector('.loading'), null);
  });

  test('should allow adding items', async ({ page }) => {
    const countBefore = await page.locator('.item').count();

    await page.locator('#input-text').fill('hello');
    await page.locator('#input-add').click();

    await page.waitForFunction(() => !document.querySelector('.loading'), null);

    expect(await page.locator('.item').count()).toBe(countBefore + 1);
  });
});