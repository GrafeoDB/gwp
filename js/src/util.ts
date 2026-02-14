/** Utility functions for gRPC client wrappers. */

import type { ServiceError } from "@grpc/grpc-js";

/**
 * Promisify a unary gRPC client method call.
 *
 * The generated grpc-js clients use callback-style APIs.
 * This wraps them in a Promise for async/await usage.
 */
export function promisifyUnary<
  TClient extends Record<string, any>,
  TMethod extends keyof TClient,
>(
  client: TClient,
  method: TMethod,
  request: Parameters<TClient[TMethod]>[0],
): Promise<any> {
  return new Promise((resolve, reject) => {
    (client[method] as any)(
      request,
      (err: ServiceError | null, response: any) => {
        if (err) reject(err);
        else resolve(response);
      },
    );
  });
}
