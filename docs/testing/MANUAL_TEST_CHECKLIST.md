# Manual Test Checklist - Skill Engine Web Interface

This checklist covers critical user workflows that should be manually verified before releases. Use this in conjunction with automated tests to ensure comprehensive coverage.

## Test Environment Setup

- [ ] Backend server running on localhost:3001
- [ ] Frontend accessible via browser
- [ ] Test skills available in working directory
- [ ] Browser DevTools console open for errors

## Browser Compatibility

Test in the following browsers:

- [ ] Chrome/Chromium (latest)
- [ ] Safari/WebKit (latest)
- [ ] Firefox (optional, bonus coverage)

## 1. Skills Browser Page

### 1.1 Initial Load

- [ ] Page loads without errors
- [ ] Skills list displays correctly
- [ ] Skill cards show all information (name, version, description, status)
- [ ] No console errors on page load

### 1.2 Search Functionality

- [ ] Search bar is visible and functional
- [ ] Typing in search bar filters skills by name
- [ ] Search is case-insensitive
- [ ] Search filters by description content
- [ ] Clearing search shows all skills again
- [ ] No results message displays when no matches

### 1.3 Filter Functionality

- [ ] Status filter dropdown works (Configured/Unconfigured/All)
- [ ] Filtering by status updates skill list correctly
- [ ] Runtime filter works (WASM/Docker/Binary/All)
- [ ] Source filter works (GitHub/Local/All)
- [ ] Multiple filters can be combined
- [ ] Filter badges display active filters
- [ ] Clear filters button works

### 1.4 Sort Functionality

- [ ] Sort by name (ascending) works
- [ ] Sort by name (descending) works
- [ ] Sort by execution count works
- [ ] Sort by last used works
- [ ] Sort persists when filtering/searching

### 1.5 Skill Cards

- [ ] Click on skill card navigates to detail page
- [ ] Skill status badge shows correct color
- [ ] Tool count displays correctly
- [ ] Instance count displays correctly
- [ ] Source information is accurate
- [ ] Hover effects work

### 1.6 Install Skill Modal

- [ ] "Install Skill" button opens modal
- [ ] Modal displays with proper styling
- [ ] Source input field works (typing)
- [ ] Runtime selection dropdown works
- [ ] Instance name input works
- [ ] "Install" button is disabled when required fields empty
- [ ] "Install" button becomes enabled when fields filled
- [ ] Installing skill shows loading state
- [ ] Success message displays after installation
- [ ] Modal closes after successful install
- [ ] New skill appears in skills list
- [ ] Cancel button closes modal without action
- [ ] Click backdrop closes modal

### 1.7 Import Config Modal

- [ ] "Import Config" button opens modal
- [ ] TOML editor textarea works
- [ ] Invalid TOML shows error message
- [ ] Valid TOML enables import button
- [ ] Import button shows loading state
- [ ] Success message after import
- [ ] Imported skills appear in list
- [ ] Cancel/close works correctly

## 2. Run Page

### 2.1 Initial State

- [ ] Page loads without errors
- [ ] Skill selector dropdown populated
- [ ] Tool selector initially disabled (no skill selected)
- [ ] Execute button initially disabled
- [ ] Services section visible
- [ ] No console errors

### 2.2 Skill Selection

- [ ] Skill dropdown shows all configured skills
- [ ] Selecting skill enables tool dropdown
- [ ] Skill selection persists on page refresh (if implemented)
- [ ] Changing skill resets tool selection
- [ ] Changing skill clears previous parameters

### 2.3 Tool Selection

- [ ] Tool dropdown shows all tools for selected skill
- [ ] Tool descriptions display (if available)
- [ ] Selecting tool displays parameter form
- [ ] Tool selection persists with skill change prevention

### 2.4 Parameter Form

- [ ] All parameter fields render correctly
- [ ] String parameters accept text input
- [ ] Number parameters accept numeric input only
- [ ] Boolean parameters show checkbox/toggle
- [ ] JSON parameters show code editor (if implemented)
- [ ] Required parameters marked with asterisk
- [ ] Optional parameters clearly indicated
- [ ] Default values pre-populated
- [ ] Validation messages show for invalid input
- [ ] Execute button disabled with invalid/missing required params

### 2.5 Execution

- [ ] Execute button becomes enabled when all required params filled
- [ ] Clicking execute shows loading state
- [ ] Output section displays execution results
- [ ] Output format selector works (JSON/Raw/Pretty)
- [ ] Switching format re-renders output correctly
- [ ] Execution time displayed
- [ ] Exit code/status displayed
- [ ] Long output scrolls properly
- [ ] Multiple executions append to history (if implemented)

### 2.6 Output Display

- [ ] JSON output is syntax highlighted
- [ ] Pretty format is readable and formatted
- [ ] Raw format shows unmodified output
- [ ] Copy to clipboard button works
- [ ] Copy button shows success feedback
- [ ] Error output styled differently (red/warning)
- [ ] stdout and stderr distinguished (if separated)

### 2.7 Services Section

- [ ] Services list displays all available services
- [ ] Service status shows correctly (running/stopped)
- [ ] Start button works for stopped services
- [ ] Stop button works for running services
- [ ] Service port/URL displays when running
- [ ] Status updates without page refresh
- [ ] Error messages display for failed start/stop

## 3. Skill Detail Page

### 3.1 Skill Information

- [ ] Page loads for valid skill name
- [ ] 404 or error page for invalid skill name
- [ ] Skill name and version displayed
- [ ] Full description shown
- [ ] Source information accurate
- [ ] Runtime information correct
- [ ] Installation date/time shown (if available)

### 3.2 Tools List

- [ ] All tools for skill listed
- [ ] Tool names displayed
- [ ] Tool descriptions shown
- [ ] Parameter information visible
- [ ] Click tool navigates to Run page with pre-selection

### 3.3 Instances Section

- [ ] All instances listed
- [ ] Instance names shown
- [ ] Instance status correct
- [ ] Configuration details visible
- [ ] Actions work (delete, edit if implemented)

### 3.4 Actions

- [ ] Uninstall button works
- [ ] Uninstall shows confirmation dialog
- [ ] Confirming uninstall removes skill
- [ ] Canceling uninstall keeps skill
- [ ] Update button works (if implemented)
- [ ] Configure button works (if implemented)

## 4. Navigation

### 4.1 Header/Navigation Bar

- [ ] Logo/title displays correctly
- [ ] Navigation links work (Skills/Run/etc.)
- [ ] Active page highlighted in nav
- [ ] Navigation persists across pages
- [ ] No broken links

### 4.2 Routing

- [ ] Direct URL navigation works
- [ ] Browser back button works correctly
- [ ] Browser forward button works correctly
- [ ] Page refresh maintains state (where appropriate)
- [ ] Deep links work (e.g., /skills/skill-name)

## 5. Error Handling

### 5.1 Network Errors

- [ ] Backend disconnection shows error message
- [ ] Retry mechanism works (if implemented)
- [ ] User-friendly error messages displayed
- [ ] No raw error objects shown to user

### 5.2 Validation Errors

- [ ] Invalid form input shows clear error messages
- [ ] Required field errors are specific
- [ ] Format errors (e.g., invalid JSON) are clear
- [ ] Errors clear when input corrected

### 5.3 API Errors

- [ ] 404 errors handled gracefully
- [ ] 500 errors show appropriate message
- [ ] Authentication errors handled (if applicable)
- [ ] Timeout errors communicated clearly

## 6. Visual Regression Checks

### 6.1 Layout

- [ ] No overlapping elements
- [ ] Proper spacing and margins
- [ ] Responsive design works (if implemented)
- [ ] Mobile view acceptable (if supported)
- [ ] No horizontal scroll on standard viewport

### 6.2 Styling

- [ ] Colors consistent with design system
- [ ] Fonts render correctly
- [ ] Icons display properly
- [ ] Buttons styled consistently
- [ ] Forms have proper styling

### 6.3 Interactions

- [ ] Hover effects work smoothly
- [ ] Click feedback visible
- [ ] Loading spinners appear during async operations
- [ ] Transitions/animations smooth
- [ ] Focus states visible for accessibility

## 7. Performance Checks

### 7.1 Load Times

- [ ] Initial page load under 3 seconds
- [ ] Skill list loads quickly (< 1 second for < 50 skills)
- [ ] Navigation between pages feels instant
- [ ] No janky animations or interactions

### 7.2 Responsiveness

- [ ] UI remains responsive during execution
- [ ] Long-running operations don't freeze UI
- [ ] Large skill lists scroll smoothly
- [ ] Search/filter operations feel instant

## 8. Data Integrity

### 8.1 State Management

- [ ] Installing skill updates list immediately
- [ ] Uninstalling skill removes from list
- [ ] Execution history accurate
- [ ] Statistics (execution count) update correctly
- [ ] No stale data displayed after updates

### 8.2 Persistence

- [ ] Filters persist during session (if implemented)
- [ ] Execution history preserved (if implemented)
- [ ] User preferences saved (if implemented)

## 9. Accessibility (Bonus)

- [ ] Tab navigation works logically
- [ ] Focus visible on all interactive elements
- [ ] Form labels associated with inputs
- [ ] Alt text on images (if any)
- [ ] ARIA labels on custom components
- [ ] Keyboard shortcuts work (if implemented)

## 10. Edge Cases

### 10.1 Empty States

- [ ] No skills installed shows helpful message
- [ ] No execution history shows appropriate message
- [ ] Empty search results handled gracefully
- [ ] No services available handled properly

### 10.2 Extreme Data

- [ ] Very long skill names don't break layout
- [ ] Large number of skills (50+) handled
- [ ] Very long execution output scrolls properly
- [ ] Special characters in names/descriptions handled

### 10.3 Concurrent Operations

- [ ] Multiple simultaneous executions work
- [ ] Installing while executing doesn't break state
- [ ] Rapid clicking doesn't cause duplicate actions

## Test Execution Notes

### Before Testing

1. Ensure backend server is running: `cargo run -p skill-http`
2. Clear browser cache if testing fresh install flow
3. Open browser DevTools console
4. Prepare test skills if needed

### During Testing

1. Check console for JavaScript errors after each action
2. Monitor network tab for failed requests
3. Note any unexpected behavior, even if minor
4. Take screenshots of visual issues

### After Testing

1. Document all issues found
2. Categorize by severity (Critical/High/Medium/Low)
3. Create GitHub issues for bugs
4. Update this checklist if new scenarios discovered

## Critical Path Summary

If time is limited, focus on these critical paths:

1. **Install and execute a skill end-to-end**
   - Install skill → Navigate to Run page → Select skill/tool → Fill parameters → Execute → View output

2. **Search and filter workflow**
   - Open Skills page → Search for skill → Apply filters → Sort results → Open skill detail

3. **Service management**
   - Navigate to Run page → Start required service → Execute tool that uses service → Verify success

4. **Error handling**
   - Attempt invalid operations → Verify error messages are clear and actionable

## Sign-Off

- [ ] All critical tests passing
- [ ] No console errors in any browser
- [ ] All blocking bugs resolved
- [ ] Release notes updated with known issues

**Tested by**: _______________
**Date**: _______________
**Browser versions**: _______________
**Build/Version**: _______________
**Notes**: _______________
