#!/usr/bin/env python3
import urllib.request
import urllib.error
import json
import re
import os
import sys

def get_github_data(url, token=None):
    req = urllib.request.Request(url)
    req.add_header('User-Agent', 'frost-tune-badge-updater')
    if token:
        req.add_header('Authorization', f'token {token}')
    try:
        with urllib.request.urlopen(req) as response:
            return response.read(), response.headers
    except urllib.error.URLError as e:
        print(f"Error fetching {url}: {e}", file=sys.stderr)
        return None, None

def main():
    token = os.environ.get('GITHUB_TOKEN')
    owner = "bukutsu"
    repo = "frost-tune"
    
    # Fetch repository details (stars, forks, open issues)
    repo_url = f"https://api.github.com/repos/{owner}/{repo}"
    repo_data_bytes, _ = get_github_data(repo_url, token)
    if not repo_data_bytes:
        print("Failed to fetch repository details from GitHub API.", file=sys.stderr)
        sys.exit(1)
        
    repo_data = json.loads(repo_data_bytes.decode('utf-8'))
    stars = repo_data.get('stargazers_count', 0)
    forks = repo_data.get('forks_count', 0)
    issues = repo_data.get('open_issues_count', 0)
    
    # Fetch contributors count
    contrib_url = f"https://api.github.com/repos/{owner}/{repo}/contributors?per_page=1"
    _, headers = get_github_data(contrib_url, token)
    
    contributors = 1 # fallback
    if headers:
        link_header = headers.get('Link', '')
        if link_header:
            match = re.search(r'page=(\d+)>; rel="last"', link_header)
            if match:
                contributors = int(match.group(1))
            else:
                all_contrib_url = f"https://api.github.com/repos/{owner}/{repo}/contributors?per_page=100"
                all_bytes, _ = get_github_data(all_contrib_url, token)
                if all_bytes:
                    all_data = json.loads(all_bytes.decode('utf-8'))
                    contributors = len(all_data)
        else:
            all_contrib_url = f"https://api.github.com/repos/{owner}/{repo}/contributors?per_page=100"
            all_bytes, _ = get_github_data(all_contrib_url, token)
            if all_bytes:
                all_data = json.loads(all_bytes.decode('utf-8'))
                contributors = len(all_data)

    print(f"Stats fetched - Stars: {stars}, Forks: {forks}, Issues: {issues}, Contributors: {contributors}")
    
    # Read README.md
    readme_path = os.path.join(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))), "README.md")
    with open(readme_path, "r", encoding="utf-8") as f:
        content = f.read()
        
    # Replace the shields
    replacements = {
        r'\[contributors-shield\]:\s*https://img\.shields\.io/\S+': 
            f'[contributors-shield]: https://img.shields.io/badge/contributors-{contributors}-blue?style=flat&logo=github',
        r'\[forks-shield\]:\s*https://img\.shields\.io/\S+': 
            f'[forks-shield]: https://img.shields.io/badge/forks-{forks}-blue?style=flat&logo=github',
        r'\[stars-shield\]:\s*https://img\.shields\.io/\S+': 
            f'[stars-shield]: https://img.shields.io/badge/stars-{stars}-brightgreen?style=flat&logo=github',
        r'\[issues-shield\]:\s*https://img\.shields\.io/\S+': 
            f'[issues-shield]: https://img.shields.io/badge/issues-{issues}%20open-important?style=flat&logo=github',
        r'\[license-shield\]:\s*https://img\.shields\.io/\S+': 
            f'[license-shield]: https://img.shields.io/badge/license-MIT-brightgreen?style=flat&logo=github'
    }
    
    modified = False
    new_content = content
    for pattern, replacement in replacements.items():
        new_content, count = re.subn(pattern, replacement, new_content)
        if count > 0:
            modified = True
            
    if modified:
        if new_content != content:
            with open(readme_path, "w", encoding="utf-8") as f:
                f.write(new_content)
            print("Successfully updated README.md with static badges.")
        else:
            print("README.md already has correct static badges.")
    else:
        print("Failed to find badge definitions in README.md to replace.", file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    main()
