/**
 * Winner Selection Panel for Competition Mode
 *
 * Hybrid approach: AI evaluates and recommends top 3, user confirms the winner
 */
import React, { useState, useEffect } from 'react';
import type { WorktreeSession, EvaluationScore, CompetitionEvaluation } from '@/types/worktree';
import { evaluateCompetition, formatScore, getRankDisplay } from '@/utils/competitionEvaluator';
import { Trophy, Award, CheckCircle, Info } from 'lucide-react';
import { useSessionHistoryStore } from '@/store/sessionHistoryStore';

interface WinnerSelectionPanelProps {
  session: WorktreeSession;
}

export const WinnerSelectionPanel: React.FC<WinnerSelectionPanelProps> = ({ session }) => {
  const [evaluation, setEvaluation] = useState<CompetitionEvaluation | null>(null);
  const [selectedWinnerId, setSelectedWinnerId] = useState<number | null>(null);
  const [isConfirming, setIsConfirming] = useState(false);
  const { updateSession } = useSessionHistoryStore();

  // Evaluate on mount or when instances change
  useEffect(() => {
    if (session.type === 'competition' && session.instances.length > 0) {
      // Use cached evaluation if available
      if (session.evaluation) {
        setEvaluation(session.evaluation);
        setSelectedWinnerId(session.winnerId || session.evaluation.recommendedWinnerId);
      } else {
        // Perform new evaluation
        const newEvaluation = evaluateCompetition(session.id, session.instances);
        setEvaluation(newEvaluation);
        setSelectedWinnerId(newEvaluation.recommendedWinnerId);
      }
    }
  }, [session]);

  const handleConfirmWinner = async () => {
    if (!selectedWinnerId || !evaluation) return;

    setIsConfirming(true);
    try {
      // Update session with winner and evaluation
      const updatedInstances = session.instances.map((instance) => ({
        ...instance,
        status: (instance.instanceId === selectedWinnerId ? instance.status : 'archived') as any,
      }));

      await updateSession({
        ...session,
        winnerId: selectedWinnerId,
        evaluation,
        status: 'completed',
        instances: updatedInstances,
      });

      alert(`✅ Winner confirmed: Instance #${selectedWinnerId}`);
    } catch (error) {
      console.error('Failed to confirm winner:', error);
      alert('Failed to confirm winner. Please try again.');
    } finally {
      setIsConfirming(false);
    }
  };

  if (!evaluation) {
    return (
      <div className="p-6 text-center text-gray-500">
        <Info className="w-8 h-8 mx-auto mb-2" />
        <p>Evaluating competition results...</p>
      </div>
    );
  }

  const isWinnerConfirmed = session.winnerId !== undefined;

  return (
    <div className="p-6 space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <Trophy className="w-6 h-6 text-yellow-500" />
          <h3 className="text-xl font-bold text-gray-900">Winner Selection</h3>
        </div>
        {isWinnerConfirmed && (
          <div className="flex items-center gap-2 px-4 py-2 bg-green-100 rounded-lg">
            <CheckCircle className="w-5 h-5 text-green-600" />
            <span className="text-sm font-semibold text-green-800">
              Winner Confirmed: Instance #{session.winnerId}
            </span>
          </div>
        )}
      </div>

      {/* Recommended Winner Banner */}
      {!isWinnerConfirmed && (
        <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
          <div className="flex items-start gap-3">
            <Award className="w-5 h-5 text-blue-600 flex-shrink-0 mt-0.5" />
            <div>
              <p className="font-semibold text-blue-900 text-sm">AI Recommendation</p>
              <p className="text-xs text-blue-700 mt-1">
                Based on automated evaluation, Instance #{evaluation.recommendedWinnerId} is recommended.
                Review the scores below and select your winner.
              </p>
            </div>
          </div>
        </div>
      )}

      {/* Score Cards */}
      <div className="space-y-3">
        {evaluation.scores.map((score) => (
          <ScoreCard
            key={score.instanceId}
            score={score}
            isSelected={selectedWinnerId === score.instanceId}
            isWinner={session.winnerId === score.instanceId}
            onSelect={() => !isWinnerConfirmed && setSelectedWinnerId(score.instanceId)}
            disabled={isWinnerConfirmed}
          />
        ))}
      </div>

      {/* Confirm Button */}
      {!isWinnerConfirmed && (
        <div className="flex justify-end gap-3 pt-4 border-t">
          <button
            onClick={handleConfirmWinner}
            disabled={!selectedWinnerId || isConfirming}
            className="
              px-6 py-3 rounded-lg font-semibold
              bg-blue-600 text-white
              hover:bg-blue-700
              disabled:opacity-50 disabled:cursor-not-allowed
              transition-colors
            "
          >
            {isConfirming ? 'Confirming...' : `Confirm Instance #${selectedWinnerId} as Winner`}
          </button>
        </div>
      )}
    </div>
  );
};

/**
 * Score Card Component
 */
interface ScoreCardProps {
  score: EvaluationScore;
  isSelected: boolean;
  isWinner: boolean;
  onSelect: () => void;
  disabled: boolean;
}

const ScoreCard: React.FC<ScoreCardProps> = ({
  score,
  isSelected,
  isWinner,
  onSelect,
  disabled,
}) => {
  return (
    <button
      onClick={onSelect}
      disabled={disabled}
      className={`
        w-full text-left p-4 rounded-lg border-2 transition-all
        ${isWinner
          ? 'bg-green-50 border-green-500'
          : isSelected
          ? 'bg-blue-50 border-blue-500'
          : score.isRecommended
          ? 'bg-yellow-50 border-yellow-300 hover:border-yellow-400'
          : 'bg-white border-gray-200 hover:border-gray-300'
        }
        ${disabled ? 'cursor-default' : 'cursor-pointer'}
      `}
    >
      <div className="flex items-start justify-between mb-3">
        <div className="flex items-center gap-3">
          <span className="text-2xl">{getRankDisplay(score.rank)}</span>
          <div>
            <div className="font-semibold text-gray-900">
              {score.agentName} #{score.instanceId}
            </div>
            <div className="text-xs text-gray-600">
              Runtime: {score.runtime}
            </div>
          </div>
        </div>
        <div className="text-right">
          <div className="text-2xl font-bold text-blue-600">
            {formatScore(score.totalScore)}
          </div>
          {score.isRecommended && !isWinner && (
            <div className="text-xs text-yellow-700 font-medium">
              ⭐ Recommended
            </div>
          )}
          {isWinner && (
            <div className="text-xs text-green-700 font-medium">
              🏆 Winner
            </div>
          )}
        </div>
      </div>

      {/* Score Breakdown */}
      <div className="grid grid-cols-4 gap-2 text-xs">
        <div>
          <div className="text-gray-600">Tests</div>
          <div className="font-semibold">{formatScore(score.scoreBreakdown.testScore)}</div>
        </div>
        <div>
          <div className="text-gray-600">Quality</div>
          <div className="font-semibold">{formatScore(score.scoreBreakdown.complexityScore)}</div>
        </div>
        <div>
          <div className="text-gray-600">Speed</div>
          <div className="font-semibold">{formatScore(score.scoreBreakdown.efficiencyScore)}</div>
        </div>
        <div>
          <div className="text-gray-600">Changes</div>
          <div className="font-semibold">{formatScore(score.scoreBreakdown.changeScore)}</div>
        </div>
      </div>

      {/* Metrics */}
      <div className="mt-3 pt-3 border-t border-gray-200 text-xs text-gray-600">
        Files: {score.metrics.filesChanged} |
        Lines: +{score.metrics.linesAdded} -{score.metrics.linesDeleted} |
        Time: {score.metrics.executionTime}s
        {score.metrics.testsPassed !== undefined && (
          <> | Tests: {score.metrics.testsPassed}/{score.metrics.testsPassed + (score.metrics.testsFailed || 0)}</>
        )}
      </div>
    </button>
  );
};
