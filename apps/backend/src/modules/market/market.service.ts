import { Injectable } from '@nestjs/common';
import { ConfigService } from '@nestjs/config';
import { HttpService } from '@nestjs/axios';
import { InjectModel } from '@nestjs/mongoose';
import { Model } from 'mongoose';
import { EpsGrowth } from '@epsx/shared';
import { Paginate } from '@epsx/shared';
import { UserRole } from '../../shared/guards/role.guard';

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

  async getEpsGrowth(skip: number, limit: number, userRole: UserRole = UserRole.PUBLIC): Promise<Paginate<EpsGrowth>> {
    // Calculate base skip and limit based on role restrictions
    let rankLimit: number;
    switch (userRole) {
      case UserRole.PREMIUM:
        rankLimit = 1;
        break;
      case UserRole.BASIC:
        rankLimit = 11;
        break;
      default:
        rankLimit = 21;
    }

    // First, get total count
    const total = await this.epsGrowthModel.countDocuments();

    // Calculate real skip based on rank limit
    const effectiveSkip = Math.max(0, skip + (rankLimit - 1));

    // Get paginated data
    const data = await this.epsGrowthModel
      .find()
      .sort({ eps_growth: -1 })
      .skip(effectiveSkip)
      .limit(limit)
      .lean()
      .exec();

    // Add rank to each document
    const rankedData = data.map((doc, index) => ({
      ...doc,
      rank: effectiveSkip + index + 1
    }));

    // Calculate available documents after rank filter
    const availableTotal = Math.max(0, total - (rankLimit - 1));

    return {
      data: rankedData,
      total: availableTotal,
      page: Math.floor(skip / limit) + 1,
      limit,
    };
  }
}
