// Node.js types excluded from browser build - only browser-compatible exports below

export interface ContractConfig {
  specUrl: string;
  baseUrl: string;
  endpoints?: string[];
}

export interface ContractResult {
  passed: boolean;
  failures: string[];
  summary: { total: number; passed: number; failed: number };
}

export interface FuzzConfig {
  url: string;
  method?: 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH';
  headers?: Record<string, string>;
  fields?: Record<string, { type: string; maxLength?: number; min?: number; max?: number; pattern?: string }>;
  iterations?: number;
}

export interface FuzzResult {
  passed: number;
  total: number;
  failures: Array<{ input: Record<string, unknown>; status: number; response: string }>;
}

export interface TestContractConfig {
  baseUrl: string;
  contracts: Array<{
    endpoint: string;
    method?: string;
    body?: Record<string, unknown>;
    expectedStatus: number;
    responseSchema?: Record<string, unknown>;
  }>;
}

export async function validateContract(config: ContractConfig): Promise<ContractResult> {
  const result: ContractResult = {
    passed: true,
    failures: [],
    summary: { total: 0, passed: 0, failed: 0 },
  };

  const endpoints = config.endpoints || [];

  for (const ep of endpoints) {
    result.summary.total++;
    try {
      const [method, path] = ep.split(' ');
      const url = `${config.baseUrl}${path}`;
      const response = await fetch(url, { method, headers: { Accept: 'application/json' } });

      if (response.status >= 200 && response.status < 400) {
        result.summary.passed++;
      } else {
        result.passed = false;
        result.summary.failed++;
        result.failures.push(`${ep} returned ${response.status}`);
      }
    } catch (err) {
      result.passed = false;
      result.summary.failed++;
      result.failures.push(`${ep} failed: ${(err as Error).message}`);
    }
  }

  return result;
}

export async function testContract(config: TestContractConfig): Promise<ContractResult> {
  const result: ContractResult = {
    passed: true,
    failures: [],
    summary: { total: config.contracts.length, passed: 0, failed: 0 },
  };

  for (const contract of config.contracts) {
    try {
      const url = `${config.baseUrl}${contract.endpoint.replace(/^(GET|POST|PUT|DELETE|PATCH)\s+/, '')}`;
      const method = contract.method || 'GET';

      const response = await fetch(url, {
        method,
        headers: { 'Content-Type': 'application/json' },
        body: contract.body ? JSON.stringify(contract.body) : undefined,
      });

      if (response.status === contract.expectedStatus) {
        result.summary.passed++;

        if (contract.responseSchema) {
          const data = await response.json();
          const valid = validateSchema(data, contract.responseSchema);
          if (!valid) {
            result.passed = false;
            result.summary.failed++;
            result.failures.push(`${method} ${contract.endpoint}: response schema mismatch`);
          }
        }
      } else {
        result.passed = false;
        result.summary.failed++;
        result.failures.push(`${method} ${contract.endpoint}: expected ${contract.expectedStatus}, got ${response.status}`);
      }
    } catch (err) {
      result.passed = false;
      result.summary.failed++;
      result.failures.push(`${contract.endpoint} failed: ${(err as Error).message}`);
    }
  }

  return result;
}

export async function fuzzEndpoint(config: FuzzConfig): Promise<FuzzResult> {
  const result: FuzzResult = { passed: 0, total: 0, failures: [] };
  const iterations = config.iterations || 100;
  const method = config.method || 'POST';

  for (let i = 0; i < iterations; i++) {
    result.total++;
    const payload = generateFuzzPayload(config.fields || {});

    try {
      const response = await fetch(config.url, {
        method,
        headers: { 'Content-Type': 'application/json', ...config.headers },
        body: JSON.stringify(payload),
      });

      // Fuzz test passes if server doesn't crash (returns any HTTP response)
      if (response.status < 500) {
        result.passed++;
      } else {
        result.failures.push({
          input: payload,
          status: response.status,
          response: await response.text(),
        });
      }
    } catch {
      result.failures.push({ input: payload, status: 0, response: 'Connection failed' });
    }
  }

  return result;
}

function validateSchema(data: unknown, schema: Record<string, unknown>): boolean {
  if (!data || typeof data !== 'object') return false;
  if (schema.type === 'object' && schema.properties) {
    const props = schema.properties as Record<string, { type: string }>;
    return Object.keys(props).every((key) => {
      const val = (data as Record<string, unknown>)[key];
      return val !== undefined && typeof val === props[key].type;
    });
  }
  return true;
}

function generateFuzzPayload(fields: Record<string, { type: string; maxLength?: number; min?: number; max?: number; pattern?: string }>): Record<string, unknown> {
  const payload: Record<string, unknown> = {};

  for (const [key, config] of Object.entries(fields)) {
    switch (config.type) {
      case 'string':
        if (config.pattern === 'email') {
          const emails = ['valid@test.com', '', 'not-an-email', 'a@b', 'user@domain.co.uk', '<script>alert(1)</script>', "' OR '1'='1"];
          payload[key] = emails[Math.floor(Math.random() * emails.length)];
        } else if (config.maxLength) {
          const lengths = [0, 1, config.maxLength - 1, config.maxLength, config.maxLength + 1, config.maxLength * 10];
          const len = lengths[Math.floor(Math.random() * lengths.length)];
          payload[key] = 'A'.repeat(Math.max(0, len));
        } else {
          const strings = ['', 'normal', 'unicode: 你好世界', 'x'.repeat(10000), "' OR 1=1 --", '<script>xss</script>', null as unknown as string, undefined as unknown as string];
          payload[key] = strings[Math.floor(Math.random() * strings.length)];
        }
        break;
      case 'number':
        const numbers = [0, -1, 1, config.min ?? -1000 - 1, config.max ?? 1000 + 1, NaN, Infinity, 0.0000001, Number.MAX_SAFE_INTEGER];
        payload[key] = numbers[Math.floor(Math.random() * numbers.length)];
        break;
      case 'boolean':
        payload[key] = Math.random() > 0.5;
        break;
      case 'array':
        payload[key] = [[], [1], Array(1000).fill('x'), null as unknown as unknown[]];
        payload[key] = (payload[key] as unknown[][])[Math.floor(Math.random() * 4)];
        break;
      case 'object':
        payload[key] = [{}, null, { nested: { deep: true } }][Math.floor(Math.random() * 3)];
        break;
      default:
        payload[key] = null;
    }
  }

  return payload;
}

export function generateLoadTest(config: {
  target: string;
  phases: Array<{ duration: number; arrivalRate: number }>;
  flow: Array<{ get?: string; post?: { url: string; json: Record<string, unknown> } }>;
}): string {
  const yaml = `config:
  target: "${config.target}"
  phases:
${config.phases.map((p) => `    - duration: ${p.duration}\n      arrivalRate: ${p.arrivalRate}`).join('\n')}
  defaults:
    headers:
      Content-Type: "application/json"
scenarios:
  - flow:
${config.flow
  .map((step) => {
    if (step.get) {
      return `      - get:\n          url: "${step.get}"`;
    }
    if (step.post) {
      return `      - post:\n          url: "${step.post.url}"\n          json: ${JSON.stringify(step.post.json, null, 10).replace(/^/gm, '            ')}`;
    }
    return '';
  })
  .join('\n')}
`;
  return yaml;
}

export interface BenchmarkResult {
  latency: { p50: number; p95: number; p99: number; max: number };
  rps: number;
  requests: number;
  errors: number;
  duration: number;
}

export function parseBenchmark(raw: string): BenchmarkResult {
  const lines = raw.split('\n');
  let rps = 0, requests = 0, errors = 0, duration = 0;
  let p50 = 0, p95 = 0, p99 = 0, max = 0;

  for (const line of lines) {
    const rpsMatch = line.match(/Requests\/sec[:\s]+([\d.]+)/);
    if (rpsMatch) rps = parseFloat(rpsMatch[1]);

    const reqMatch = line.match(/(\d+)\s+requests in/);
    if (reqMatch) requests = parseInt(reqMatch[1]);

    const errMatch = line.match(/(\d+)\s+errors/);
    if (errMatch) errors = parseInt(errMatch[1]);

    const durMatch = line.match(/in\s+([\d.]+)s/);
    if (durMatch) duration = parseFloat(durMatch[1]);

    const p50Match = line.match(/p50[:\s]+([\d.]+)/);
    if (p50Match) p50 = parseFloat(p50Match[1]);

    const p95Match = line.match(/p95[:\s]+([\d.]+)/);
    if (p95Match) p95 = parseFloat(p95Match[1]);

    const p99Match = line.match(/p99[:\s]+([\d.]+)/);
    if (p99Match) p99 = parseFloat(p99Match[1]);

    const maxMatch = line.match(/max[:\s]+([\d.]+)/);
    if (maxMatch) max = parseFloat(maxMatch[1]);
  }

  return { latency: { p50, p95, p99, max }, rps, requests, errors, duration };
}
