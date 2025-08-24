#!/usr/bin/env python3
"""
Test script for the URL Generator Lambda function
Tests various scenarios including success cases and error conditions
"""

import boto3
import json
import time
import sys
import argparse
from datetime import datetime

def test_lambda_function(function_name, region, environment, project_name):
    """Test the Lambda function with various scenarios"""
    
    lambda_client = boto3.client('lambda', region_name=region)
    
    print(f"Testing Lambda function: {function_name}")
    print(f"Region: {region}")
    print(f"Environment: {environment}")
    print("=" * 50)
    
    # Test cases
    test_cases = [
        {
            "name": "Valid firmware request",
            "payload": {
                "device_id": f"{project_name}-{environment}-test-001",
                "request_type": "firmware",
                "resource_id": "1.0.0",
                "request_id": f"test-fw-{int(time.time())}"
            },
            "expected_status": 200  # May be 404 if firmware doesn't exist
        },
        {
            "name": "Valid program request",
            "payload": {
                "device_id": f"{project_name}-{environment}-test-002",
                "request_type": "program",
                "resource_id": "sensor-monitor-v1",
                "request_id": f"test-prog-{int(time.time())}"
            },
            "expected_status": 200  # May be 404 if program doesn't exist
        },
        {
            "name": "Invalid device ID",
            "payload": {
                "device_id": "invalid-device-id",
                "request_type": "firmware",
                "resource_id": "1.0.0",
                "request_id": f"test-invalid-{int(time.time())}"
            },
            "expected_status": 403
        },
        {
            "name": "Missing parameters",
            "payload": {
                "device_id": f"{project_name}-{environment}-test-003",
                "request_type": "firmware"
                # Missing resource_id
            },
            "expected_status": 500
        },
        {
            "name": "Invalid request type",
            "payload": {
                "device_id": f"{project_name}-{environment}-test-004",
                "request_type": "invalid_type",
                "resource_id": "1.0.0",
                "request_id": f"test-invalid-type-{int(time.time())}"
            },
            "expected_status": 500
        }
    ]
    
    results = []
    
    for i, test_case in enumerate(test_cases, 1):
        print(f"\nTest {i}: {test_case['name']}")
        print(f"Payload: {json.dumps(test_case['payload'], indent=2)}")
        
        try:
            # Invoke Lambda function
            response = lambda_client.invoke(
                FunctionName=function_name,
                Payload=json.dumps(test_case['payload'])
            )
            
            # Parse response
            response_payload = json.loads(response['Payload'].read())
            status_code = response_payload.get('statusCode', 500)
            
            print(f"Status Code: {status_code}")
            print(f"Response: {json.dumps(response_payload, indent=2)}")
            
            # Check if status matches expectation (allowing for 404 on missing resources)
            if status_code == test_case['expected_status'] or (test_case['expected_status'] == 200 and status_code == 404):
                result = "PASS"
                print(f"Result: ✅ {result}")
            else:
                result = "FAIL"
                print(f"Result: ❌ {result} (Expected {test_case['expected_status']}, got {status_code})")
            
            results.append({
                'test': test_case['name'],
                'result': result,
                'status_code': status_code,
                'expected': test_case['expected_status']
            })
            
        except Exception as e:
            print(f"Error: {str(e)}")
            print(f"Result: ❌ FAIL (Exception)")
            results.append({
                'test': test_case['name'],
                'result': 'FAIL',
                'error': str(e)
            })
        
        print("-" * 30)
    
    # Summary
    print(f"\n{'='*50}")
    print("TEST SUMMARY")
    print(f"{'='*50}")
    
    passed = sum(1 for r in results if r['result'] == 'PASS')
    total = len(results)
    
    for result in results:
        status_icon = "✅" if result['result'] == 'PASS' else "❌"
        print(f"{status_icon} {result['test']}: {result['result']}")
    
    print(f"\nTotal: {passed}/{total} tests passed")
    
    return passed == total

def test_s3_security_policies(firmware_bucket, programs_bucket, region):
    """Test S3 bucket security policies"""
    
    s3_client = boto3.client('s3', region_name=region)
    
    print(f"\nTesting S3 Security Policies")
    print(f"Firmware Bucket: {firmware_bucket}")
    print(f"Programs Bucket: {programs_bucket}")
    print("=" * 50)
    
    test_results = []
    
    # Test 1: Check public access is blocked
    for bucket_name in [firmware_bucket, programs_bucket]:
        try:
            public_access = s3_client.get_public_access_block(Bucket=bucket_name)
            config = public_access['PublicAccessBlockConfiguration']
            
            all_blocked = all([
                config['BlockPublicAcls'],
                config['BlockPublicPolicy'],
                config['IgnorePublicAcls'],
                config['RestrictPublicBuckets']
            ])
            
            if all_blocked:
                print(f"✅ {bucket_name}: Public access properly blocked")
                test_results.append(True)
            else:
                print(f"❌ {bucket_name}: Public access not fully blocked")
                test_results.append(False)
                
        except Exception as e:
            print(f"❌ {bucket_name}: Error checking public access block: {e}")
            test_results.append(False)
    
    # Test 2: Check encryption is enabled
    for bucket_name in [firmware_bucket, programs_bucket]:
        try:
            encryption = s3_client.get_bucket_encryption(Bucket=bucket_name)
            rules = encryption['ServerSideEncryptionConfiguration']['Rules']
            
            if rules and rules[0]['ApplyServerSideEncryptionByDefault']['SSEAlgorithm']:
                print(f"✅ {bucket_name}: Encryption enabled")
                test_results.append(True)
            else:
                print(f"❌ {bucket_name}: Encryption not properly configured")
                test_results.append(False)
                
        except Exception as e:
            print(f"❌ {bucket_name}: Error checking encryption: {e}")
            test_results.append(False)
    
    # Test 3: Check versioning is enabled
    for bucket_name in [firmware_bucket, programs_bucket]:
        try:
            versioning = s3_client.get_bucket_versioning(Bucket=bucket_name)
            
            if versioning.get('Status') == 'Enabled':
                print(f"✅ {bucket_name}: Versioning enabled")
                test_results.append(True)
            else:
                print(f"❌ {bucket_name}: Versioning not enabled")
                test_results.append(False)
                
        except Exception as e:
            print(f"❌ {bucket_name}: Error checking versioning: {e}")
            test_results.append(False)
    
    passed = sum(test_results)
    total = len(test_results)
    print(f"\nS3 Security Tests: {passed}/{total} passed")
    
    return passed == total

def main():
    parser = argparse.ArgumentParser(description='Test Lambda function and S3 security policies')
    parser.add_argument('--environment', '-e', default='dev', help='Environment (dev, staging, prod)')
    parser.add_argument('--region', '-r', default='us-west-2', help='AWS region')
    parser.add_argument('--project-name', '-p', default='esp32-steel', help='Project name')
    
    args = parser.parse_args()
    
    # Get stack outputs
    cf_client = boto3.client('cloudformation', region_name=args.region)
    
    try:
        # Get S3 and Lambda stack outputs
        s3_stack_name = f"{args.project_name}-{args.environment}-s3-lambda"
        s3_stack = cf_client.describe_stacks(StackName=s3_stack_name)
        
        outputs = {output['OutputKey']: output['OutputValue'] 
                  for output in s3_stack['Stacks'][0]['Outputs']}
        
        function_name = outputs['URLGeneratorFunctionName']
        firmware_bucket = outputs['FirmwareBucketName']
        programs_bucket = outputs['SteelProgramsBucketName']
        
    except Exception as e:
        print(f"Error getting stack outputs: {e}")
        print("Make sure the S3-Lambda stack is deployed")
        sys.exit(1)
    
    # Run tests
    lambda_tests_passed = test_lambda_function(
        function_name, args.region, args.environment, args.project_name
    )
    
    s3_tests_passed = test_s3_security_policies(
        firmware_bucket, programs_bucket, args.region
    )
    
    # Overall result
    print(f"\n{'='*50}")
    print("OVERALL TEST RESULTS")
    print(f"{'='*50}")
    
    if lambda_tests_passed and s3_tests_passed:
        print("✅ All tests passed!")
        sys.exit(0)
    else:
        print("❌ Some tests failed!")
        sys.exit(1)

if __name__ == '__main__':
    main()