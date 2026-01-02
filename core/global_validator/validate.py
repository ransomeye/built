#!/usr/bin/env python3
# Path and File Name : /home/ransomeye/rebuild/core/global_validator/validate.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: CLI entry point for Global Forensic Consistency Validator

"""
Global Forensic Consistency Validator - CLI Entry Point

Usage:
    python3 validate.py [--json] [--output OUTPUT_FILE]

Exit codes:
    0: Validation passed
    1: Validation failed (violations detected)
    2: Error during validation
"""

import sys
import json
import argparse
from pathlib import Path

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from core.global_validator.validator import GlobalForensicValidator


def main():
    """Main CLI entry point."""
    parser = argparse.ArgumentParser(
        description='Global Forensic Consistency Validator',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
    python3 validate.py                    # Run validation, print JSON to stdout
    python3 validate.py --output report.json  # Save report to file
    python3 validate.py --json | jq          # Pretty-print JSON output
        """
    )
    
    parser.add_argument(
        '--output', '-o',
        type=str,
        help='Output file for JSON report (default: stdout)'
    )
    
    parser.add_argument(
        '--project-root',
        type=str,
        default='/home/ransomeye/rebuild',
        help='Project root directory (default: /home/ransomeye/rebuild)'
    )
    
    parser.add_argument(
        '--json',
        action='store_true',
        help='Output JSON format (default: always JSON)'
    )
    
    args = parser.parse_args()
    
    try:
        # Create validator
        validator = GlobalForensicValidator(project_root=Path(args.project_root))
        
        # Run validation
        result = validator.validate_all()
        
        # Generate report
        report = validator.generate_report(result)
        
        # Output report
        report_json = json.dumps(report, indent=2)
        
        if args.output:
            # Write to file
            output_path = Path(args.output)
            output_path.parent.mkdir(parents=True, exist_ok=True)
            with open(output_path, 'w') as f:
                f.write(report_json)
            print(f"Validation report written to: {output_path}", file=sys.stderr)
        else:
            # Print to stdout
            print(report_json)
        
        # Exit with appropriate code
        if result.passed:
            return 0
        else:
            return 1
    
    except Exception as e:
        print(f"Error during validation: {e}", file=sys.stderr)
        import traceback
        traceback.print_exc()
        return 2


if __name__ == '__main__':
    sys.exit(main())

