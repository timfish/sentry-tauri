import { createServer, type IncomingMessage } from 'node:http';
import type { AddressInfo } from 'node:net';
import { gunzipSync } from 'node:zlib';
import { type Envelope, parseEnvelope } from '@sentry/core';

export interface TestServer {
  readonly dsn: string;
  close(): Promise<void>;
}

async function readBody(req: IncomingMessage): Promise<Buffer> {
  const chunks: Buffer[] = [];
  for await (const chunk of req) {
    chunks.push(chunk);
  }
  const body = Buffer.concat(chunks);
  return req.headers['content-encoding'] === 'gzip' ? gunzipSync(body) : body;
}

/**
 * Parses an envelope and decodes the JSON of every item that is not an
 * attachment. `parseEnvelope` leaves items with a `length` header as bytes.
 */
function parse(body: Buffer): Envelope {
  const [headers, items] = parseEnvelope(body);
  const decoded = items.map(([header, payload]) =>
    header.type !== 'attachment' && payload instanceof Uint8Array
      ? [header, JSON.parse(Buffer.from(payload).toString('utf8'))]
      : [header, payload],
  );
  return [headers, decoded] as Envelope;
}

/**
 * A mock Sentry server that accepts envelopes and passes them to `callback`.
 */
export async function createSentryTestServer(callback: (envelope: Envelope) => void): Promise<TestServer> {
  const server = createServer(async (req, res) => {
    const url = req.url || '';
    if (req.method !== 'POST' || !/^\/api\/\d+\/envelope\//.test(url)) {
      res.writeHead(404).end();
      return;
    }

    const auth = (req.headers['x-sentry-auth'] as string | undefined) || url;
    if (!/sentry_key=\w+/.test(auth)) {
      res.writeHead(403).end('Missing authentication header');
      return;
    }

    try {
      callback(parse(await readBody(req)));
      res.writeHead(200).end('Success');
    } catch (error) {
      console.error('Failed to parse envelope', error);
      res.writeHead(400).end();
    }
  });

  await new Promise<void>((resolve) => server.listen(0, '127.0.0.1', resolve));
  const { port } = server.address() as AddressInfo;

  return {
    dsn: `http://public@127.0.0.1:${port}/1`,
    close: () =>
      new Promise((resolve, reject) => {
        server.closeAllConnections();
        server.close((error) => (error ? reject(error) : resolve()));
      }),
  };
}
