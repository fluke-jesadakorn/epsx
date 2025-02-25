import { createParamDecorator, ExecutionContext } from '@nestjs/common';

// Get user from request decorator
export const CurrentUser = createParamDecorator(
  (data: unknown, ctx: ExecutionContext) => {
    const request = ctx.switchToHttp().getRequest();
    return request.user;
  },
);

// API Version decorator
export const ApiVersion = (version: string) =>
  createParamDecorator((data: unknown, ctx: ExecutionContext) => {
    const request = ctx.switchToHttp().getRequest();
    request.apiVersion = version;
    return request;
  });

// Public route decorator (skip auth)
export const Public = () => {
  return (target: any, propertyKey?: string, descriptor?: PropertyDescriptor) => {
    if (descriptor) {
      Reflect.defineMetadata('isPublic', true, descriptor.value);
      return descriptor;
    }
    Reflect.defineMetadata('isPublic', true, target);
    return target;
  };
};
