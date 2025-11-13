/**
 * Competition Evaluator
 *
 * Automatic evaluation logic for Competition mode instances
 * Calculates scores based on test results, code quality, efficiency, and changes
 */
import type { WorktreeInstance, EvaluationScore, EvaluationMetrics, CompetitionEvaluation } from '@/types/worktree';

/**
 * Evaluate all instances in a competition and return scored results
 */
export function evaluateCompetition(
  competitionId: string,
  instances: WorktreeInstance[]
): CompetitionEvaluation {
  const scores: EvaluationScore[] = instances.map((instance) => {
    const metrics = extractMetrics(instance);
    const scoreBreakdown = calculateScoreBreakdown(metrics);
    const totalScore =
      scoreBreakdown.testScore +
      scoreBreakdown.complexityScore +
      scoreBreakdown.efficiencyScore +
      scoreBreakdown.changeScore;

    return {
      instanceId: instance.instanceId,
      agentName: instance.agentName,
      runtime: instance.runtime || 'unknown',
      totalScore,
      metrics,
      rank: 0, // Will be assigned after sorting
      isRecommended: false, // Will be assigned to top 3
      scoreBreakdown,
    };
  });

  // Sort by total score (descending)
  scores.sort((a, b) => b.totalScore - a.totalScore);

  // Assign ranks and recommendations
  scores.forEach((score, index) => {
    score.rank = index + 1;
    score.isRecommended = index < 3; // Top 3 are recommended
  });

  return {
    competitionId,
    evaluatedAt: new Date().toISOString(),
    scores,
    recommendedWinnerId: scores[0]?.instanceId || 0,
  };
}

/**
 * Extract metrics from instance
 */
function extractMetrics(instance: WorktreeInstance): EvaluationMetrics {
  const filesChanged = instance.filesChanged || 0;
  const linesAdded = instance.linesAdded || 0;
  const linesDeleted = instance.linesDeleted || 0;
  const executionTime = instance.executionTime || calculateExecutionTime(instance);

  // Calculate success rate based on status
  const successRate = instance.status === 'completed' ? 100 :
                      instance.status === 'failed' ? 0 : 50;

  return {
    filesChanged,
    linesAdded,
    linesDeleted,
    executionTime,
    testsPassed: instance.testsPassed,
    testsFailed: instance.testsFailed,
    codeComplexity: instance.codeComplexity,
    successRate,
  };
}

/**
 * Calculate execution time from startTime and endTime if executionTime is not set
 */
function calculateExecutionTime(instance: WorktreeInstance): number {
  if (!instance.startTime || !instance.endTime) {
    return 0;
  }
  const start = new Date(instance.startTime).getTime();
  const end = new Date(instance.endTime).getTime();
  return Math.floor((end - start) / 1000); // Convert to seconds
}

/**
 * Calculate score breakdown (total 100 points)
 */
function calculateScoreBreakdown(metrics: EvaluationMetrics) {
  // 1. Test Score (0-40 points)
  const testScore = calculateTestScore(metrics);

  // 2. Complexity Score (0-30 points)
  const complexityScore = calculateComplexityScore(metrics);

  // 3. Efficiency Score (0-20 points)
  const efficiencyScore = calculateEfficiencyScore(metrics);

  // 4. Change Score (0-10 points)
  const changeScore = calculateChangeScore(metrics);

  return { testScore, complexityScore, efficiencyScore, changeScore };
}

/**
 * Test Score: Based on success rate and test results
 * 0-40 points
 */
function calculateTestScore(metrics: EvaluationMetrics): number {
  const baseScore = (metrics.successRate / 100) * 40;

  // Bonus if we have actual test results
  if (metrics.testsPassed !== undefined && metrics.testsFailed !== undefined) {
    const total = metrics.testsPassed + metrics.testsFailed;
    if (total > 0) {
      const testPassRate = (metrics.testsPassed / total) * 100;
      return (testPassRate / 100) * 40;
    }
  }

  return baseScore;
}

/**
 * Complexity Score: Lower complexity is better
 * 0-30 points
 */
function calculateComplexityScore(metrics: EvaluationMetrics): number {
  if (metrics.codeComplexity === undefined) {
    // No complexity data, use moderate score
    return 15;
  }

  // Complexity 0-100, where 0 is best
  // Convert to score where lower complexity = higher score
  const complexityPenalty = metrics.codeComplexity / 100;
  return (1 - complexityPenalty) * 30;
}

/**
 * Efficiency Score: Based on execution time
 * 0-20 points
 */
function calculateEfficiencyScore(metrics: EvaluationMetrics): number {
  if (metrics.executionTime === 0) {
    return 10; // Moderate score if no timing data
  }

  // Assume 300 seconds (5 min) is average, 600s (10 min) is max
  const avgTime = 300;
  const maxTime = 600;

  if (metrics.executionTime <= avgTime) {
    // Faster than average: 10-20 points
    const bonus = (avgTime - metrics.executionTime) / avgTime;
    return 10 + (bonus * 10);
  } else if (metrics.executionTime <= maxTime) {
    // Slower than average: 0-10 points
    const penalty = (metrics.executionTime - avgTime) / (maxTime - avgTime);
    return 10 - (penalty * 10);
  } else {
    // Too slow: 0 points
    return 0;
  }
}

/**
 * Change Score: Reasonable amount of changes
 * 0-10 points
 */
function calculateChangeScore(metrics: EvaluationMetrics): number {
  const totalLines = metrics.linesAdded + metrics.linesDeleted;

  // Ideal range: 50-500 lines changed
  const minIdeal = 50;
  const maxIdeal = 500;

  if (totalLines >= minIdeal && totalLines <= maxIdeal) {
    return 10; // Perfect score
  } else if (totalLines < minIdeal) {
    // Too few changes (might be incomplete)
    return (totalLines / minIdeal) * 10;
  } else {
    // Too many changes (might be over-engineered)
    const excessPenalty = Math.min((totalLines - maxIdeal) / maxIdeal, 1);
    return 10 - (excessPenalty * 5);
  }
}

/**
 * Format score as percentage string
 */
export function formatScore(score: number): string {
  return `${Math.round(score)}%`;
}

/**
 * Get rank display with medal emoji
 */
export function getRankDisplay(rank: number): string {
  switch (rank) {
    case 1: return '🥇 1st';
    case 2: return '🥈 2nd';
    case 3: return '🥉 3rd';
    default: return `#${rank}`;
  }
}
