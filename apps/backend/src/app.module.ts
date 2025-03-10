// Main application module
import { Module, MiddlewareConsumer, NestModule } from "@nestjs/common";
import { APP_GUARD } from "@nestjs/core";
import { ConfigModule, ConfigService } from "@nestjs/config";
import { MongooseModule } from "@nestjs/mongoose";
import * as cookieParser from "cookie-parser";
import { AuthModule } from "./modules/auth/auth.module";
import { MarketModule } from "./modules/market/market.module";
import { FinancialModule } from "./modules/financial/financial.module";
import { StockModule } from "./modules/stock/stock.module";
import { SharedModule } from "./shared/shared.module";
import { DatabaseConfigService } from "./shared/services/database-config.service";
import { FirebaseAuthGuard } from "./shared/guards/role.guard";

@Module({
  imports: [
    ConfigModule.forRoot({
      isGlobal: true,
      envFilePath: '.env',
    }),
    MongooseModule.forRootAsync({
      imports: [ConfigModule],
      useClass: DatabaseConfigService,
    }),
    SharedModule,
    AuthModule,
    MarketModule,
    FinancialModule,
    StockModule,
  ],
  providers: [
    {
      provide: APP_GUARD,
      useClass: FirebaseAuthGuard,
    },
  ],
})
export class AppModule implements NestModule {
  constructor(private configService: ConfigService) {}

  configure(consumer: MiddlewareConsumer) {
    // Use cookie parser middleware
    consumer.apply(cookieParser()).forRoutes("*");

    // Log only non-sensitive parts of configuration
    const backendUrl = this.configService.get("BACKEND_URL");
    const frontendUrl = this.configService.get("FRONTEND_URL");
    const mongoDbUri = this.configService.get("MONGODB_URI");

    if (process.env.NODE_ENV !== 'production') {
      console.log("Backend URL:", backendUrl);
      console.log("Frontend URL:", frontendUrl);
      // Only log the host part of MongoDB URI, not the credentials
      console.log("MongoDB Host:", mongoDbUri?.split('@')[1] || 'Not configured');
    }
  }
}
