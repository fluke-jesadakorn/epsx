import { Injectable } from '@nestjs/common';
import { ConfigService } from '@nestjs/config';
import { HttpService } from '@nestjs/axios';
import { InjectModel } from '@nestjs/mongoose';
import { Model } from 'mongoose';
import { EpsGrowth } from '@epsx/shared';
import { Paginate } from '@epsx/shared';

@Injectable()
export class MarketService {
  constructor(
    private readonly configService: ConfigService,
    private readonly httpService: HttpService,
    @InjectModel(EpsGrowth.name)
    private readonly epsGrowthModel: Model<EpsGrowth>,
  ) {}

  async getHealth() {
    return {
      status: 'ok',
      timestamp: new Date().toISOString(),
    };
  }

  async getEpsGrowth(skip: number, limit: number): Promise<Paginate<EpsGrowth>> {
    const [data, total] = await Promise.all([
      this.epsGrowthModel
        .find()
        .sort({ eps_growth: -1 })
        .skip(skip)
        .limit(limit)
        .exec(),
      this.epsGrowthModel.countDocuments(),
    ]);

    return {
      data,
      total,
      page: Math.floor(skip / limit) + 1,
      limit,
    };
  }
}
