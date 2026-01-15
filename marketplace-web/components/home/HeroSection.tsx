interface HeroSectionProps {
  stats: {
    totalSkills: number;
    totalTools: number;
    byType: Record<string, number>;
    officialCount: number;
    featuredCount: number;
  };
}

export function HeroSection({ stats }: HeroSectionProps) {
  return (
    <div className="relative py-20 lg:py-32 overflow-hidden">
      {/* Glow Effects */}
      <div className="absolute top-0 left-1/2 -translate-x-1/2 w-full h-full max-w-7xl opacity-40 pointer-events-none">
        <div className="absolute top-1/4 left-1/4 w-96 h-96 bg-primary-500/20 rounded-full blur-3xl animate-pulse-slow" />
        <div className="absolute bottom-1/4 right-1/4 w-96 h-96 bg-indigo-500/20 rounded-full blur-3xl animate-pulse-slow delay-1000" />
      </div>

      <div className="relative max-w-5xl mx-auto text-center px-4">
        <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-white/5 border border-white/10 text-sm text-gray-300 mb-8 animate-fade-in">
          <span className="flex h-2 w-2 rounded-full bg-green-500 animate-pulse"></span>
          Production Ready Skills
        </div>

        <h1 className="text-5xl md:text-7xl font-bold tracking-tight text-white mb-6 animate-enter text-balance">
          Supercharge your agents with <span className="text-gradient-primary">Working Skills</span>
        </h1>

        <p className="text-xl text-gray-400 mb-10 max-w-2xl mx-auto leading-relaxed animate-enter [animation-delay:200ms] text-balance">
          A curated library of {stats.totalSkills} self-contained skills for AI agents.
          Enhance your workflows with WASM, Native, and Docker runtime capabilities.
        </p>

        {/* Stats Grid */}
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4 max-w-3xl mx-auto animate-enter [animation-delay:400ms]">
          {[
            { label: 'Total Skills', value: stats.totalSkills },
            { label: 'Cloud Tools', value: stats.totalTools },
            { label: 'WASM Runtime', value: stats.byType.wasm || 0 },
            { label: 'Official', value: stats.officialCount || 0 }, // Using officialCount from stats
          ].map((stat, i) => (
            <div key={i} className="glass-card p-4 flex flex-col items-center justify-center group hover:bg-white/5 transition-colors">
              <span className="text-3xl font-bold text-white mb-1 group-hover:scale-110 transition-transform duration-300">{stat.value}</span>
              <span className="text-sm text-gray-500 uppercase tracking-wider font-medium">{stat.label}</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
