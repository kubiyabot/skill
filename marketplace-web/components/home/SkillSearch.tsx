/**
 * Real-time search bar component with dropdown results
 * Features: keyboard navigation, debouncing, click-outside handling
 */

'use client';

import { useState, useMemo, useCallback, useRef, useEffect } from 'react';
import { Search, X } from 'lucide-react';
import { useRouter, useSearchParams } from 'next/navigation';
import Fuse from 'fuse.js';
import { SkillCardData } from '@/lib/types';
import { createSkillSearchIndex, searchSkills } from '@/lib/search/skillSearch';
import { cn } from '@/lib/utils/cn';

interface SkillSearchProps {
  skills: SkillCardData[];
  onResultClick?: (skill: SkillCardData) => void;
  placeholder?: string;
  maxResults?: number;
}

export function SkillSearch({
  skills,
  onResultClick,
  placeholder = 'Search skills, tools, categories...',
  maxResults = 8,
}: SkillSearchProps) {
  const router = useRouter();
  const searchParams = useSearchParams();
  const [query, setQuery] = useState(searchParams.get('q') || '');
  const [isOpen, setIsOpen] = useState(false);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const searchRef = useRef<HTMLDivElement>(null);

  // Create search index (memoized based on skills)
  const searchIndex = useMemo(
    () => createSkillSearchIndex(skills),
    [skills]
  );

  // Search results (memoized based on query and search index)
  const results = useMemo(() => {
    if (!query.trim()) return [];
    return searchSkills(searchIndex, query).slice(0, maxResults);
  }, [searchIndex, query, maxResults]);

  // Handle keyboard navigation
  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent<HTMLInputElement>) => {
      if (e.key === 'ArrowDown') {
        e.preventDefault();
        setSelectedIndex((prev) => Math.min(prev + 1, results.length - 1));
      } else if (e.key === 'ArrowUp') {
        e.preventDefault();
        setSelectedIndex((prev) => Math.max(prev - 1, 0));
      } else if (e.key === 'Enter' && results[selectedIndex]) {
        e.preventDefault();
        handleResultClick(results[selectedIndex]);
      } else if (e.key === 'Escape') {
        setIsOpen(false);
        setQuery('');
      }
    },
    [results, selectedIndex]
  );

  // Handle result click
  const handleResultClick = useCallback(
    (skill: SkillCardData) => {
      if (onResultClick) {
        onResultClick(skill);
      } else {
        router.push(`/skills/${skill.slug}`);
      }
      setIsOpen(false);
      setQuery('');
    },
    [onResultClick, router]
  );

  // Handle input change with debounced URL update
  const handleInputChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const value = e.target.value;
      setQuery(value);
      setIsOpen(true);
      setSelectedIndex(0);

      // Debounced URL update for filtering the main grid
      const timeoutId = setTimeout(() => {
        const params = new URLSearchParams(window.location.search);
        if (value.trim()) {
          params.set('q', value);
        } else {
          params.delete('q');
        }
        router.push(`?${params.toString()}`, { scroll: false });
      }, 500);

      return () => clearTimeout(timeoutId);
    },
    [router]
  );

  // Handle clear button
  const handleClear = useCallback(() => {
    setQuery('');
    setSelectedIndex(0);
    setIsOpen(false);

    // Remove search query from URL
    const params = new URLSearchParams(window.location.search);
    params.delete('q');
    router.push(`?${params.toString()}`, { scroll: false });
  }, [router]);

  // Close dropdown on click outside
  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (
        searchRef.current &&
        !searchRef.current.contains(e.target as Node)
      ) {
        setIsOpen(false);
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  return (
    <div ref={searchRef} className="relative w-full max-w-2xl mx-auto">
      {/* Search Input */}
      <div className="relative group">
        <div className="absolute inset-0 bg-primary-500/20 rounded-xl blur-xl opacity-0 group-hover:opacity-100 transition-opacity duration-500" />
        <Search className="absolute left-4 top-1/2 -translate-y-1/2 h-5 w-5 text-gray-500 group-hover:text-primary-400 transition-colors" />
        <input
          type="text"
          value={query}
          onChange={handleInputChange}
          onFocus={() => query && setIsOpen(true)}
          onKeyDown={handleKeyDown}
          placeholder={placeholder}
          className="relative w-full pl-12 pr-12 py-4 text-base bg-dark-800/80 border border-white/10 text-gray-100 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary-500/50 focus:border-transparent placeholder-gray-500 shadow-xl backdrop-blur-sm transition-all duration-200"
          aria-label="Search skills"
          aria-autocomplete="list"
          aria-controls="search-results"
          aria-expanded={isOpen && results.length > 0}
        />
        {query && (
          <button
            onClick={handleClear}
            className="absolute right-4 top-1/2 -translate-y-1/2 text-gray-500 hover:text-white transition-colors p-1"
            aria-label="Clear search"
            type="button"
          >
            <X className="h-4 w-4" />
          </button>
        )}
      </div>

      {/* Dropdown Results */}
      {isOpen && results.length > 0 && (
        <div
          id="search-results"
          className="absolute top-full left-0 right-0 mt-3 bg-dark-800 border border-white/10 rounded-xl shadow-2xl max-h-96 overflow-y-auto z-50 animate-enter"
          role="listbox"
        >
          {results.map((skill, index) => (
            <button
              key={skill.id}
              onClick={() => handleResultClick(skill)}
              onMouseEnter={() => setSelectedIndex(index)}
              className={cn(
                "w-full px-4 py-3.5 text-left flex items-start gap-4 transition-colors",
                index === selectedIndex ? 'bg-white/5' : 'hover:bg-white/5',
                index !== results.length - 1 ? 'border-b border-white/5' : ''
              )}
              role="option"
              aria-selected={index === selectedIndex}
              type="button"
            >
              {/* Icon */}
              {skill.icon ? (
                <div className="flex-shrink-0 w-10 h-10 rounded-lg bg-dark-900 border border-white/10 flex items-center justify-center p-2">
                  <img
                    src={skill.icon}
                    alt=""
                    className="w-full h-full object-contain"
                  />
                </div>
              ) : (
                <div className="flex-shrink-0 w-10 h-10 rounded-lg bg-white/5" />
              )}

              {/* Content */}
              <div className="flex-grow min-w-0">
                <div className="flex items-center gap-2 mb-1">
                  <p className="font-semibold text-gray-100 truncate">
                    {skill.name}
                  </p>
                  <span className="inline-flex items-center px-1.5 py-0.5 rounded text-[10px] font-mono font-medium bg-white/10 text-gray-400 border border-white/5 uppercase tracking-wider">
                    {skill.type}
                  </span>
                </div>
                <p className="text-sm text-gray-500 line-clamp-1">
                  {skill.description}
                </p>
                <p className="text-xs text-primary-400/80 mt-1.5 font-medium">
                  {skill.toolsCount} tools
                </p>
              </div>
            </button>
          ))}
        </div>
      )}

      {/* No Results */}
      {isOpen && query && results.length === 0 && (
        <div className="absolute top-full left-0 right-0 mt-3 bg-dark-800 border border-white/10 rounded-xl shadow-2xl p-6 z-50 text-center animate-enter">
          <p className="text-sm text-gray-400">
            No skills found for <span className="text-white font-medium">&quot;{query}&quot;</span>
          </p>
        </div>
      )}
    </div>
  );
}
