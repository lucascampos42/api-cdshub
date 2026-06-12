import { Injectable, NotFoundException } from '@nestjs/common';
import { PrismaService } from '../prisma/prisma.service';
import { CreateSuggestionDto } from './dto/suggestion.dto';
import { SuggestionStatus } from '@prisma/client';

@Injectable()
export class SuggestionsService {
  constructor(private prisma: PrismaService) {}

  async findAll(system?: string, page: number = 1, limit: number = 10) {
    const skip = (page - 1) * limit;
    const where = system ? { system } : {};

    const [items, total] = await Promise.all([
      this.prisma.suggestion.findMany({
        where,
        orderBy: { votes: 'desc' },
        skip,
        take: limit,
        include: {
          createdBy: {
            select: {
              id: true,
              name: true,
            },
          },
        },
      }),
      this.prisma.suggestion.count({ where }),
    ]);

    return {
      items,
      meta: {
        total,
        page,
        limit,
        totalPages: Math.ceil(total / limit),
      },
    };
  }

  async create(createSuggestionDto: CreateSuggestionDto, userId?: string) {
    return this.prisma.suggestion.create({
      data: {
        ...createSuggestionDto,
        status: SuggestionStatus.ABERTO,
        createdById: userId,
      },
    });
  }

  async vote(id: string) {
    const suggestion = await this.prisma.suggestion.findUnique({
      where: { id },
    });

    if (!suggestion) {
      throw new NotFoundException('Sugestão não encontrada');
    }

    return this.prisma.suggestion.update({
      where: { id },
      data: {
        votes: {
          increment: 1,
        },
      },
    });
  }

  async updateStatus(id: string, status: SuggestionStatus) {
    return this.prisma.suggestion.update({
      where: { id },
      data: { status },
    });
  }
}
