#!/usr/bin/env python3
"""
Aurelia Agent Monitoring and Validation Script
This script monitors deployed agents and validates their autonomous behavior
"""

import json
import subprocess
import time
import sys
import os
from datetime import datetime
from typing import Dict, List, Tuple
import argparse

class AureliaMonitor:
    def __init__(self, config_file: str):
        with open(config_file, 'r') as f:
            self.config = json.load(f)
        self.test_results = {
            'start_time': datetime.now().isoformat(),
            'tests': [],
            'summary': {}
        }
    
    def execute_ssh_command(self, server: str, command: str) -> Tuple[bool, str]:
        """Execute command on remote server via SSH"""
        try:
            result = subprocess.run(
                ['ssh', f'ubuntu@{server}', command],
                capture_output=True,
                text=True,
                timeout=30
            )
            return result.returncode == 0, result.stdout + result.stderr
        except subprocess.TimeoutExpired:
            return False, "Command timed out"
        except Exception as e:
            return False, str(e)
    
    def test_agent_running(self, server: Dict) -> Dict:
        """Test if agent is running on server"""
        test_result = {
            'name': 'agent_running',
            'server': server['ip'],
            'timestamp': datetime.now().isoformat()
        }
        
        cmd = f"cd {server['remote_deploy_path']} && ps aux | grep kernel | grep -v grep"
        success, output = self.execute_ssh_command(server['ip'], cmd)
        
        test_result['passed'] = success
        test_result['output'] = output[:500]  # Limit output size
        
        return test_result
    
    def test_resource_usage(self, server: Dict) -> Dict:
        """Test resource usage is within limits"""
        test_result = {
            'name': 'resource_usage',
            'server': server['ip'],
            'timestamp': datetime.now().isoformat()
        }
        
        limits = self.config['test_config']['resource_limits']
        
        # Check CPU usage
        cmd = "top -bn1 | grep 'Cpu(s)' | awk '{print $2}' | cut -d'%' -f1"
        success, cpu_output = self.execute_ssh_command(server['ip'], cmd)
        
        if success:
            try:
                cpu_usage = float(cpu_output.strip())
                test_result['cpu_usage'] = cpu_usage
                test_result['cpu_passed'] = cpu_usage < limits['max_cpu_percent']
            except:
                test_result['cpu_passed'] = False
        
        # Check memory usage
        cmd = f"cd {server['remote_deploy_path']} && pgrep -f kernel | xargs -I {{}} pmap {{}} | tail -1"
        success, mem_output = self.execute_ssh_command(server['ip'], cmd)
        
        if success and 'total' in mem_output.lower():
            try:
                mem_kb = int(mem_output.split()[1].replace('K', ''))
                mem_mb = mem_kb / 1024
                test_result['memory_mb'] = mem_mb
                test_result['memory_passed'] = mem_mb < limits['max_memory_mb']
            except:
                test_result['memory_passed'] = False
        
        test_result['passed'] = test_result.get('cpu_passed', False) and \
                               test_result.get('memory_passed', False)
        
        return test_result
    
    def test_log_activity(self, server: Dict) -> Dict:
        """Test if agent is producing logs"""
        test_result = {
            'name': 'log_activity',
            'server': server['ip'],
            'timestamp': datetime.now().isoformat()
        }
        
        cmd = f"cd {server['remote_deploy_path']} && tail -n 50 aurelia.log | grep -E '\\[(INFO|WARN|ERROR)\\]' | wc -l"
        success, output = self.execute_ssh_command(server['ip'], cmd)
        
        if success:
            try:
                log_lines = int(output.strip())
                test_result['log_lines'] = log_lines
                test_result['passed'] = log_lines > 0
            except:
                test_result['passed'] = False
        else:
            test_result['passed'] = False
        
        test_result['output'] = output[:500]
        
        return test_result
    
    def test_self_replication(self, primary_server: str, replica_server: str) -> Dict:
        """Test self-replication capability"""
        test_result = {
            'name': 'self_replication',
            'primary': primary_server,
            'replica': replica_server,
            'timestamp': datetime.now().isoformat()
        }
        
        # Check if replica has been deployed
        cmd = "test -f /home/ubuntu/aurelia_replica/kernel && echo 'EXISTS' || echo 'NOT_FOUND'"
        success, output = self.execute_ssh_command(replica_server, cmd)
        
        test_result['passed'] = success and 'EXISTS' in output
        test_result['output'] = output.strip()
        
        if test_result['passed']:
            # Check if replica is running
            cmd = "cd /home/ubuntu/aurelia_replica && ps aux | grep kernel | grep -v grep"
            success, output = self.execute_ssh_command(replica_server, cmd)
            test_result['replica_running'] = success
        
        return test_result
    
    def test_network_communication(self, servers: List[Dict]) -> Dict:
        """Test network communication between agents"""
        test_result = {
            'name': 'network_communication',
            'timestamp': datetime.now().isoformat(),
            'connections': []
        }
        
        for server in servers:
            # Check WebSocket connections
            cmd = f"netstat -an | grep ':8080' | grep ESTABLISHED | wc -l"
            success, output = self.execute_ssh_command(server['ip'], cmd)
            
            if success:
                try:
                    connections = int(output.strip())
                    test_result['connections'].append({
                        'server': server['ip'],
                        'websocket_connections': connections
                    })
                except:
                    pass
        
        test_result['passed'] = len(test_result['connections']) > 0
        
        return test_result
    
    def test_autonomous_behavior(self, server: Dict) -> Dict:
        """Test autonomous decision-making behavior"""
        test_result = {
            'name': 'autonomous_behavior',
            'server': server['ip'],
            'timestamp': datetime.now().isoformat()
        }
        
        # Check for strategy decisions in logs
        cmd = f"cd {server['remote_deploy_path']} && grep -E 'StrategyDecision|DECISION' aurelia.log | tail -n 10"
        success, output = self.execute_ssh_command(server['ip'], cmd)
        
        test_result['has_decisions'] = success and len(output.strip()) > 0
        
        # Check for perception events
        cmd = f"cd {server['remote_deploy_path']} && grep -E 'MarketData|Perception' aurelia.log | tail -n 10"
        success, output = self.execute_ssh_command(server['ip'], cmd)
        
        test_result['has_perception'] = success and len(output.strip()) > 0
        
        # Check for reasoning events
        cmd = f"cd {server['remote_deploy_path']} && grep -E 'Reasoning|Analysis' aurelia.log | tail -n 10"
        success, output = self.execute_ssh_command(server['ip'], cmd)
        
        test_result['has_reasoning'] = success and len(output.strip()) > 0
        
        test_result['passed'] = (test_result.get('has_decisions', False) or 
                                test_result.get('has_perception', False) or 
                                test_result.get('has_reasoning', False))
        
        return test_result
    
    def run_validation_suite(self):
        """Run complete validation suite"""
        print("Starting Aurelia Agent Validation Suite")
        print("=" * 50)
        
        servers = self.config['test_environments']
        primary = next((s for s in servers if s['role'] == 'primary'), None)
        replica = next((s for s in servers if s['role'] == 'replica'), None)
        
        # Run tests
        tests_to_run = [
            ('Agent Running', lambda s: self.test_agent_running(s)),
            ('Resource Usage', lambda s: self.test_resource_usage(s)),
            ('Log Activity', lambda s: self.test_log_activity(s)),
            ('Autonomous Behavior', lambda s: self.test_autonomous_behavior(s))
        ]
        
        for test_name, test_func in tests_to_run:
            print(f"\n{test_name} Test:")
            for server in servers:
                result = test_func(server)
                self.test_results['tests'].append(result)
                status = "✓ PASSED" if result.get('passed', False) else "✗ FAILED"
                print(f"  {server['name']}: {status}")
                if not result.get('passed', False):
                    print(f"    Details: {result.get('output', 'No output')[:100]}")
        
        # Test replication if both servers exist
        if primary and replica:
            print(f"\nSelf-Replication Test:")
            result = self.test_self_replication(primary['ip'], replica['ip'])
            self.test_results['tests'].append(result)
            status = "✓ PASSED" if result['passed'] else "✗ FAILED"
            print(f"  {status}")
        
        # Test network communication
        print(f"\nNetwork Communication Test:")
        result = self.test_network_communication(servers)
        self.test_results['tests'].append(result)
        status = "✓ PASSED" if result['passed'] else "✗ FAILED"
        print(f"  {status}")
        
        # Generate summary
        self.generate_summary()
        
    def generate_summary(self):
        """Generate test summary"""
        total_tests = len(self.test_results['tests'])
        passed_tests = sum(1 for t in self.test_results['tests'] if t.get('passed', False))
        
        self.test_results['summary'] = {
            'total_tests': total_tests,
            'passed': passed_tests,
            'failed': total_tests - passed_tests,
            'success_rate': (passed_tests / total_tests * 100) if total_tests > 0 else 0,
            'end_time': datetime.now().isoformat()
        }
        
        print("\n" + "=" * 50)
        print("VALIDATION SUMMARY")
        print("=" * 50)
        print(f"Total Tests: {total_tests}")
        print(f"Passed: {passed_tests}")
        print(f"Failed: {total_tests - passed_tests}")
        print(f"Success Rate: {self.test_results['summary']['success_rate']:.1f}%")
        
        # Save results to file
        with open('validation_results.json', 'w') as f:
            json.dump(self.test_results, f, indent=2)
        print(f"\nDetailed results saved to: validation_results.json")
    
    def continuous_monitoring(self, duration_minutes: int = 60):
        """Run continuous monitoring for specified duration"""
        print(f"Starting continuous monitoring for {duration_minutes} minutes")
        end_time = time.time() + (duration_minutes * 60)
        
        while time.time() < end_time:
            remaining = int((end_time - time.time()) / 60)
            print(f"\n[{datetime.now().strftime('%H:%M:%S')}] Monitoring... ({remaining} minutes remaining)")
            
            self.run_validation_suite()
            
            # Wait before next check
            time.sleep(self.config['test_config']['health_check_interval_seconds'])
        
        print("\nContinuous monitoring completed")

def main():
    parser = argparse.ArgumentParser(description='Aurelia Agent Monitoring and Validation')
    parser.add_argument('--config', default='test_env.json', help='Path to configuration file')
    parser.add_argument('--continuous', type=int, help='Run continuous monitoring for N minutes')
    parser.add_argument('--test', choices=['all', 'running', 'resources', 'logs', 'replication', 'network', 'autonomous'],
                       default='all', help='Specific test to run')
    
    args = parser.parse_args()
    
    if not os.path.exists(args.config):
        print(f"Error: Configuration file {args.config} not found")
        sys.exit(1)
    
    monitor = AureliaMonitor(args.config)
    
    if args.continuous:
        monitor.continuous_monitoring(args.continuous)
    else:
        monitor.run_validation_suite()

if __name__ == "__main__":
    main()