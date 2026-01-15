import { GITHUB_REPO } from '@/lib/utils/constants';

export function Footer() {
  return (
    <footer className="border-t border-gray-200 bg-gray-50 mt-20">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
        <div className="grid grid-cols-1 md:grid-cols-4 gap-8">
          <div className="col-span-1 md:col-span-2">
            <h3 className="text-base font-semibold text-gray-900 mb-3">
              Skills Marketplace
            </h3>
            <p className="text-sm text-gray-600 max-w-md leading-relaxed">
              Browse production-ready, self-contained agentic skills. WASM,
              Native, and Docker runtime skills for DevOps, cloud, APIs, and
              more.
            </p>
          </div>

          <div>
            <h4 className="text-sm font-semibold text-gray-900 mb-3">
              Resources
            </h4>
            <ul className="space-y-2">
              <li>
                <a
                  href="https://skill.dev"
                  className="text-sm text-gray-600 hover:text-gray-900 transition-colors"
                >
                  Documentation
                </a>
              </li>
              <li>
                <a
                  href={GITHUB_REPO}
                  className="text-sm text-gray-600 hover:text-gray-900 transition-colors"
                >
                  GitHub
                </a>
              </li>
              <li>
                <a
                  href={`${GITHUB_REPO}/tree/main/marketplace`}
                  className="text-sm text-gray-600 hover:text-gray-900 transition-colors"
                >
                  Contribute
                </a>
              </li>
            </ul>
          </div>

          <div>
            <h4 className="text-sm font-semibold text-gray-900 mb-3">
              Community
            </h4>
            <ul className="space-y-2">
              <li>
                <a
                  href={`${GITHUB_REPO}/discussions`}
                  className="text-sm text-gray-600 hover:text-gray-900 transition-colors"
                >
                  Discussions
                </a>
              </li>
              <li>
                <a
                  href={`${GITHUB_REPO}/issues`}
                  className="text-sm text-gray-600 hover:text-gray-900 transition-colors"
                >
                  Issues
                </a>
              </li>
              <li>
                <a
                  href="https://twitter.com/kubiyabot"
                  className="text-sm text-gray-600 hover:text-gray-900 transition-colors"
                >
                  Twitter
                </a>
              </li>
            </ul>
          </div>
        </div>

        <div className="border-t border-gray-200 mt-8 pt-8 text-center">
          <p className="text-sm text-gray-500">
            &copy; {new Date().getFullYear()} Skill Engine. Open source under
            MIT License.
          </p>
        </div>
      </div>
    </footer>
  );
}
