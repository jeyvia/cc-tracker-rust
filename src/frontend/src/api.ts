import axios from 'axios';

// Change this to your backend URL when deployed
const API_BASE = import.meta.env.VITE_API_URL || 'http://127.0.0.1:3000/api';

export interface AddCardRequest {
  name: string;
  categories?: string[];
  payment_categories?: string[];
  miles_per_dollar: number;
  miles_per_dollar_foreign?: number;
  block_size: number;
  renewal_date: number;
  max_reward_limit?: number;
  min_spend?: number;
}

export interface AddCardResponse {
  id: number;
  message: string;
}

export interface Card {
  id: number;
  name: string;
  categories: string;
  payment_categories: string;
  miles_per_dollar: number;
  miles_per_dollar_foreign: number | null;
  block_size: number;
  statement_renewal_date: number;
  max_reward_limit: number | null;
  min_spend: number | null;
}

export interface CardRecommendation {
  card_name: string;
  miles_per_dollar: number;
  block_size: number;
  effective_rate: number;
  miles_earned: number;
  remaining_limit: number | null;
  eligible: boolean;
  reason: string;
}

export interface Spending {
  id: number;
  card_id: number;
  amount: number;
  category: string;
  date: string;
  miles_earned: number;
}

export interface AddSpendingRequest {
  card_id: number;
  amount: number;
  category: string;
  date: string;
}

export interface AddSpendingResponse {
  id: number;
  miles_earned: number;
  message: string;
}

export const api = {
  // Cards
  async addCard(card: AddCardRequest): Promise<AddCardResponse> {
    const { data } = await axios.post(`${API_BASE}/cards`, card);
    return data;
  },

  async listCards(): Promise<Card[]> {
    const { data } = await axios.get(`${API_BASE}/cards`);
    return data;
  },

  async deleteCard(id: number): Promise<void> {
    await axios.delete(`${API_BASE}/cards?id=${id}`);
  },

  // Best Card
  async getBestCard(
    category: string,
    amount: number,
    paymentCategory: string,
    date?: string
  ): Promise<CardRecommendation[]> {
    const params = new URLSearchParams({
      category,
      amount: amount.toString(),
      payment_category: paymentCategory,
      ...(date && { date })
    });
    const { data } = await axios.get(`${API_BASE}/best-card?${params}`);
    return data;
  },

  // Spending
  async addSpending(spending: AddSpendingRequest): Promise<AddSpendingResponse> {
    const { data } = await axios.post(`${API_BASE}/spending`, spending);
    return data;
  },

  async listSpending(cardId?: number): Promise<Spending[]> {
    const params = cardId ? `?card_id=${cardId}` : '';
    const { data } = await axios.get(`${API_BASE}/spending${params}`);
    return data;
  }
};
