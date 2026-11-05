import { Button, cn } from '@repo/ui';
import { createFileRoute } from '@tanstack/react-router';
import { useFeatureState } from 'feature-react/state';
import React from 'react';
import { WindowHeader } from '@/components';
import { useSettingsCx } from '@/features/settings';
import { ProgressivePomodoroTimerCx, useTimerCx } from '@/features/timer';
import {
	completeFlowReturn,
	parseFlowReturnSearch,
	toFlowReturnTarget,
	type TFlowReturnSearch
} from '@/lib';

export const Route = createFileRoute('/window/main/progressive/rating/')({
	validateSearch: (search: Record<string, unknown>): TFlowReturnSearch => ({
		...parseFlowReturnSearch(search)
	}),
	component: RouteComponent
});

function RouteComponent() {
	const timerCx = useTimerCx<ProgressivePomodoroTimerCx>();
	const returnTarget = toFlowReturnTarget(Route.useSearch());
	const settingsCx = useSettingsCx();
	const settings = useFeatureState(settingsCx.$appSettings);

	const [selectedRatingKey, setSelectedRatingKey] = React.useState<string | null>(null);
	const [isSubmitting, setIsSubmitting] = React.useState(false);
	const [editableWork, setEditableWork] = React.useState<number[]>([]);
	const [editableBreak, setEditableBreak] = React.useState<(number | null)[]>([]);
	const [editingCell, setEditingCell] = React.useState<TEditingCell | null>(null);
	const [editingValue, setEditingValue] = React.useState<string>('');

	const ratings = settings.timer.progressive.ratings;
	const selectedRating = ratings.find((r) => r.key === selectedRatingKey) ?? null;

	// MARK: - Actions

	const handleSelectRating = React.useCallback(
		(key: string) => {
			const rating = ratings.find((r) => r.key === key);
			if (!rating) return;
			setSelectedRatingKey(key);
			setEditableWork(rating.suggestions.map((s) => s.workMinutes));
			setEditableBreak(rating.suggestions.map((s) => s.breakMinutes));
			setEditingCell(null);
		},
		[ratings]
	);

	const handleSelectSuggestion = React.useCallback(
		async (index: number, workMinutes: number, breakMinutes: number | null) => {
			if (isSubmitting || selectedRating == null) return;
			setIsSubmitting(true);

			const suggestion = selectedRating.suggestions[index];
			if (suggestion == null) {
				setIsSubmitting(false);
				return;
			}

			if (workMinutes !== suggestion.workMinutes || breakMinutes !== suggestion.breakMinutes) {
				const updatedRatings = ratings.map((r) =>
					r.key === selectedRatingKey
						? {
								...r,
								suggestions: r.suggestions.map((s, i) =>
									i === index ? { workMinutes, breakMinutes } : s
								)
							}
						: r
				);
				settingsCx.update({
					timer: {
						...settings.timer,
						progressive: { ...settings.timer.progressive, ratings: updatedRatings }
					}
				});
			}

			try {
				await timerCx.advanceWithSuggestion(
					workMinutes * 60,
					breakMinutes != null ? breakMinutes * 60 : null
				);
				await completeFlowReturn(returnTarget);
			} finally {
				setIsSubmitting(false);
			}
		},
		[
			isSubmitting,
			selectedRating,
			selectedRatingKey,
			ratings,
			returnTarget,
			settingsCx,
			settings.timer,
			timerCx
		]
	);

	const parseWork = React.useCallback(
		(raw: string) => Math.max(1, Math.min(120, parseInt(raw, 10) || 1)),
		[]
	);

	const parseBreak = React.useCallback(
		(raw: string) => Math.max(1, Math.min(60, parseInt(raw, 10) || 1)),
		[]
	);

	// MARK: - UI

	return (
		<div className="menubar-page bg-base-0 flex h-screen w-[300px] flex-col">
			<WindowHeader />

			<div className="flex flex-1 flex-col overflow-y-auto px-4 py-6">
				{selectedRating == null ? (
					// Step 1: Rate your focus
					<div className="space-y-6">
						<h1 className="text-base-900 text-xl font-semibold">How was your focus?</h1>
						<div className="flex flex-col gap-2">
							{ratings.map(({ key, label, description }) => (
								<button
									key={key}
									disabled={isSubmitting}
									onClick={() => handleSelectRating(key)}
									className="border-base-200 hover:bg-base-50 flex flex-col items-start rounded-xl border p-4 text-left transition-colors disabled:opacity-50"
								>
									<span className="text-base-900 text-sm font-semibold">{label}</span>
									<span className="text-base-400 text-xs">{description}</span>
								</button>
							))}
						</div>
					</div>
				) : (
					// Step 2: Pick next block duration
					<div className="space-y-6">
						<div>
							<h1 className="text-base-900 text-xl font-semibold">Next block</h1>
							<p className="text-base-400 mt-1 text-sm">How long do you want to focus?</p>
						</div>
						<div className="flex flex-col gap-2">
							{selectedRating.suggestions.map((suggestion, i) => {
								const workMin = editableWork[i] ?? suggestion.workMinutes;
								const breakMin =
									editableBreak[i] !== undefined ? editableBreak[i] : suggestion.breakMinutes;
								const isEditingWork = editingCell?.index === i && editingCell.field === 'work';
								const isEditingBreak = editingCell?.index === i && editingCell.field === 'break';

								return (
									<div
										key={i}
										role="button"
										tabIndex={0}
										onClick={() => handleSelectSuggestion(i, workMin, breakMin)}
										onKeyDown={(e) =>
											e.key === 'Enter' && handleSelectSuggestion(i, workMin, breakMin)
										}
										className={cn(
											'border-base-200 hover:bg-base-50 flex flex-col items-start rounded-xl border p-4 text-left transition-colors',
											isSubmitting && 'pointer-events-none opacity-50'
										)}
									>
										{/* Work duration */}
										<span
											className="text-base-900 cursor-text text-sm font-semibold"
											onClick={(e) => {
												e.stopPropagation();
												setEditingValue(String(workMin));
												setEditingCell({ index: i, field: 'work' });
											}}
										>
											{isEditingWork ? (
												<input
													type="number"
													autoFocus
													value={editingValue}
													min={1}
													max={120}
													onChange={(e) => setEditingValue(e.target.value)}
													onBlur={() => {
														const v = parseWork(editingValue);
														setEditableWork((prev) => prev.map((m, j) => (j === i ? v : m)));
														setEditingCell(null);
													}}
													onKeyDown={(e) => {
														if (e.key === 'Enter') {
															const v = parseWork(editingValue);
															setEditingCell(null);
															handleSelectSuggestion(i, v, breakMin);
														}
														e.stopPropagation();
													}}
													onClick={(e) => e.stopPropagation()}
													style={{ width: `${Math.max(1, editingValue.length)}ch` }}
													className="text-base-900 [appearance:textfield] bg-transparent text-sm font-semibold outline-none [&::-webkit-inner-spin-button]:appearance-none [&::-webkit-outer-spin-button]:appearance-none"
												/>
											) : (
												workMin
											)}{' '}
											min
										</span>

										{/* Break duration */}
										{breakMin != null ? (
											<span
												className="text-base-400 cursor-text text-xs"
												onClick={(e) => {
													e.stopPropagation();
													setEditingValue(String(breakMin));
													setEditingCell({ index: i, field: 'break' });
												}}
											>
												after{' '}
												{isEditingBreak ? (
													<input
														type="number"
														autoFocus
														value={editingValue}
														min={1}
														max={60}
														onChange={(e) => setEditingValue(e.target.value)}
														onBlur={() => {
															const v = parseBreak(editingValue);
															setEditableBreak((prev) => prev.map((m, j) => (j === i ? v : m)));
															setEditingCell(null);
														}}
														onKeyDown={(e) => {
															if (e.key === 'Enter') {
																const v = parseBreak(editingValue);
																setEditingCell(null);
																handleSelectSuggestion(i, workMin, v);
															}
															e.stopPropagation();
														}}
														onClick={(e) => e.stopPropagation()}
														style={{ width: `calc(${Math.max(1, editingValue.length)}ch + 1px)` }}
														className="text-base-400 [appearance:textfield] bg-transparent text-xs outline-none [&::-webkit-inner-spin-button]:appearance-none [&::-webkit-outer-spin-button]:appearance-none"
													/>
												) : (
													breakMin
												)}{' '}
												min break
											</span>
										) : (
											<span className="text-base-400 text-xs">after no break</span>
										)}
									</div>
								);
							})}
						</div>
					</div>
				)}
			</div>

			<footer className="border-base-200 bg-base-50 flex shrink-0 justify-between border-t px-4 py-3">
				{selectedRating != null ? (
					<Button
						variant="ghost"
						onClick={() => {
							setSelectedRatingKey(null);
							setEditableWork([]);
							setEditableBreak([]);
							setEditingCell(null);
						}}
						disabled={isSubmitting}
					>
						Back
					</Button>
				) : (
					<span />
				)}
				<Button
					variant="ghost"
					onClick={() => {
						void completeFlowReturn(returnTarget);
					}}
					disabled={isSubmitting}
				>
					Cancel
				</Button>
			</footer>
		</div>
	);
}

interface TEditingCell {
	index: number;
	field: 'work' | 'break';
}
